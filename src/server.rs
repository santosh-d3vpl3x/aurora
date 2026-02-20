use crate::distributed::DistributedPolars;
use crate::spark::connect::execute_plan_response::Response;
use crate::spark::connect::spark_connect_service_server::SparkConnectService;
use crate::spark::connect::{
    execute_plan_request::Command, ExecuteMetrics, ExecutePlanRequest, ExecutePlanResponse, Field,
    JsonRows, Schema,
};
use futures::Stream;
use polars::prelude::SchemaExt;
use serde_json::Map;
use std::pin::Pin;
use std::time::Instant;
use tonic::{Request, Response as TonicResponse, Status};

#[derive(Debug, Clone, Default)]
pub struct SparkConnectServer {
    executor: DistributedPolars,
}

#[tonic::async_trait]
impl SparkConnectService for SparkConnectServer {
    type ExecutePlanStream =
        Pin<Box<dyn Stream<Item = Result<ExecutePlanResponse, Status>> + Send>>;

    async fn execute_plan(
        &self,
        request: Request<ExecutePlanRequest>,
    ) -> Result<TonicResponse<Self::ExecutePlanStream>, Status> {
        let req = request.into_inner();
        let start = Instant::now();
        let executor = self.executor.clone();

        let stream = async_stream::try_stream! {
            let Some(Command::SqlCommand(sql)) = req.command else {
                Err(Status::invalid_argument("only sql_command is supported"))?;
                #[allow(unreachable_code)]
                return;
            };

            let df = executor.execute_sql(&sql.query, &sql.table_paths, sql.partitions as usize)
                .map_err(|e| Status::internal(e.to_string()))?;

            let fields = df.schema().iter_fields().map(|f| Field {
                name: f.name().to_string(),
                dtype: format!("{:?}", f.dtype()),
            }).collect();

            yield ExecutePlanResponse {
                response: Some(Response::Schema(Schema { fields })),
            };

            let rows: Vec<String> = (0..df.height())
                .map(|idx| {
                    let mut obj = Map::new();
                    for col in df.get_columns() {
                        let v = col.get(idx).ok().map(|x| x.to_string()).unwrap_or_else(|| "null".to_string());
                        obj.insert(col.name().to_string(), serde_json::Value::String(v));
                    }
                    serde_json::Value::Object(obj).to_string()
                })
                .collect();

            yield ExecutePlanResponse {
                response: Some(Response::Rows(JsonRows { rows })),
            };

            yield ExecutePlanResponse {
                response: Some(Response::Metrics(ExecuteMetrics {
                    elapsed_ms: start.elapsed().as_millis() as u64,
                    row_count: df.height() as u64,
                })),
            };
        };

        Ok(TonicResponse::new(
            Box::pin(stream) as Self::ExecutePlanStream
        ))
    }
}

impl SparkConnectServer {
    pub fn into_service(
        self,
    ) -> crate::spark::connect::spark_connect_service_server::SparkConnectServiceServer<Self> {
        crate::spark::connect::spark_connect_service_server::SparkConnectServiceServer::new(self)
    }
}
