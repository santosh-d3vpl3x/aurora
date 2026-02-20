use anyhow::{anyhow, Result};
use polars::prelude::*;
use serde::Serialize;

#[derive(Clone, Default)]
pub struct DistributedPolarsEngine;

#[derive(Debug, Serialize)]
pub struct QueryResult {
    pub json_rows: String,
    pub row_count: usize,
}

impl DistributedPolarsEngine {
    pub async fn execute_sql(&self, sql: &str) -> Result<QueryResult> {
        let sql = sql.trim().to_ascii_lowercase();

        // Seed a tiny in-memory table to keep this server self-contained.
        let df = df!(
            "id" => &[1_i64, 2, 3, 4],
            "value" => &[10_i64, 20, 30, 40]
        )?;

        // This models distributed work by offloading heavy operations to Polars' rayon pool.
        let lazy = match sql.as_str() {
            "select * from numbers" => df.lazy(),
            "select id, value * 2 as value2 from numbers" => df
                .lazy()
                .select([col("id"), (col("value") * lit(2)).alias("value2")]),
            _ => return Err(anyhow!("unsupported SQL in MVP engine: {sql}")),
        };

        self.collect_and_encode(lazy).await
    }

    pub async fn execute_projection(
        &self,
        values: Vec<i64>,
        add_scalar: i64,
    ) -> Result<QueryResult> {
        let values_series = Series::new("value".into(), values);
        let lazy = DataFrame::new(vec![values_series.into()])?
            .lazy()
            .with_columns([(col("value") + lit(add_scalar)).alias("projected")])
            .select([col("value"), col("projected")]);

        self.collect_and_encode(lazy).await
    }

    async fn collect_and_encode(&self, lazy: LazyFrame) -> Result<QueryResult> {
        let df = tokio::task::spawn_blocking(move || lazy.collect())
            .await
            .map_err(|e| anyhow!("polars worker join error: {e}"))??;

        let row_count = df.height();
        let json_rows = serde_json::to_string(&rows_from_df(&df)?)?;

        Ok(QueryResult {
            json_rows,
            row_count,
        })
    }
}

fn rows_from_df(df: &DataFrame) -> PolarsResult<Vec<serde_json::Value>> {
    let names = df.get_column_names_owned();
    let mut out = Vec::with_capacity(df.height());

    for idx in 0..df.height() {
        let mut obj = serde_json::Map::with_capacity(names.len());
        for name in &names {
            let col = df.column(name)?;
            let av = col.get(idx)?;
            obj.insert(name.to_string(), serde_json::Value::String(av.to_string()));
        }
        out.push(serde_json::Value::Object(obj));
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn projection_executes() {
        let engine = DistributedPolarsEngine;
        let res = engine.execute_projection(vec![1, 2, 3], 10).await.unwrap();
        assert_eq!(res.row_count, 3);
        assert!(res.json_rows.contains("projected"));
    }
}
