use anyhow::{anyhow, Context, Result};
use polars::prelude::{DataFrame, LazyFrame, ScanArgsParquet};
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
        for (table_name, path) in table_paths {
            let lazy = LazyFrame::scan_parquet(path, ScanArgsParquet::default())
                .with_context(|| format!("failed to scan parquet path pattern: {path}"))?;
            ctx.register(table_name.as_str(), lazy);
        }

        let lf = ctx.execute(query).context("polars SQL execution failed")?;
        lf.collect().context("collecting query result failed")
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
}
