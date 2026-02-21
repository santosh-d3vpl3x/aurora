use anyhow::{anyhow, Context, Result};
use polars::prelude::*;
use polars::sql::SQLContext;
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct DistributedPolars;

impl DistributedPolars {
    pub fn execute_sql(
        &self,
        query: &str,
        table_paths: &HashMap<String, String>,
        _partitions: usize,
    ) -> Result<DataFrame> {
        if query.trim().is_empty() {
            return Err(anyhow!("query cannot be empty"));
        }

        let mut ctx = SQLContext::new();

        if table_paths.is_empty() {
            // Keep an in-memory fallback relation for local MVP usage.
            let numbers = df!("id" => [1_i64, 2, 3, 4], "value" => [10_i64, 20, 30, 40])?;
            ctx.register("numbers", numbers.lazy());
        } else {
            for (table_name, path) in table_paths {
                let lazy = LazyFrame::scan_parquet(path, ScanArgsParquet::default())
                    .with_context(|| format!("failed to scan parquet path pattern: {path}"))?;
                ctx.register(table_name.as_str(), lazy);
            }
        }

        let lf = ctx.execute(query).context("polars SQL execution failed")?;
        lf.collect().context("collecting query result failed")
    }

    pub fn execute_range_repartition_count(
        &self,
        start: i64,
        end: i64,
        step: i64,
        _partitions: usize,
        _repartition: usize,
    ) -> Result<DataFrame> {
        if step == 0 {
            return Err(anyhow!("step cannot be zero"));
        }

        let mut n = 0_i64;
        if step > 0 {
            let mut v = start;
            while v < end {
                n += 1;
                v += step;
            }
        } else {
            let mut v = start;
            while v > end {
                n += 1;
                v += step;
            }
        }

        df!("count" => [n]).map_err(Into::into)
    }

    pub fn execute_projection(&self, values: Vec<i64>, add_scalar: i64) -> Result<DataFrame> {
        let values = Series::new("value".into(), values);
        DataFrame::new(vec![values.into()])
            .context("creating projection dataframe failed")
            .map(|df| {
                df.lazy()
                    .with_columns([(col("value") + lit(add_scalar)).alias("projected")])
                    .select([col("value"), col("projected")])
            })?
            .collect()
            .context("collecting projection result failed")
    }
}

#[cfg(test)]
mod tests {
    use super::DistributedPolars;
    use polars::prelude::*;
    use std::collections::HashMap;
    use tempfile::tempdir;

    #[test]
    fn executes_simple_sql() {
        let dir = tempdir().expect("tempdir");
        let p = dir.path().join("part-000.parquet");

        let mut df = df! {
            "id" => [1,2,3],
            "value" => [10,20,30]
        }
        .expect("df");

        let mut f = std::fs::File::create(&p).expect("parquet file");
        ParquetWriter::new(&mut f)
            .finish(&mut df)
            .expect("write parquet");

        let mut map = HashMap::new();
        map.insert(
            "t".to_string(),
            format!("{}/*.parquet", dir.path().display()),
        );

        let out = DistributedPolars::default()
            .execute_sql("select sum(value) as total from t", &map, 0)
            .expect("query");

        assert_eq!(out.height(), 1);
    }

    #[test]
    fn executes_range_repartition_count() {
        let out = DistributedPolars::default()
            .execute_range_repartition_count(0, 100, 1, 8, 4)
            .expect("range count");
        assert_eq!(out.height(), 1);
        assert_eq!(
            out.column("count")
                .expect("count")
                .get(0)
                .expect("row0")
                .to_string(),
            "100"
        );
    }

    #[test]
    fn executes_projection() {
        let out = DistributedPolars::default()
            .execute_projection(vec![1, 2, 3], 10)
            .expect("projection");
        assert_eq!(out.height(), 3);
        assert_eq!(
            out.column("projected")
                .expect("projected")
                .get(0)
                .expect("row0")
                .to_string(),
            "11"
        );
    }
}
