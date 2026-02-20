use std::sync::Arc;

use tonic::{Request, Response, Status};

use crate::engine::DistributedPolarsEngine;
use crate::spark::connect::{
    execute_plan_request::Plan, spark_connect_service_server::SparkConnectService,
    ExecutePlanRequest, ExecutePlanResponse,
};

#[derive(Clone)]
pub struct SparkConnectPolarsServer {
    engine: Arc<DistributedPolarsEngine>,
}

impl SparkConnectPolarsServer {
    pub fn new(engine: DistributedPolarsEngine) -> Self {
        Self {
            engine: Arc::new(engine),
        }
    }
}

#[tonic::async_trait]
impl SparkConnectService for SparkConnectPolarsServer {
    async fn execute_plan(
        &self,
        request: Request<ExecutePlanRequest>,
    ) -> Result<Response<ExecutePlanResponse>, Status> {
        let req = request.into_inner();

        let plan = req
            .plan
            .ok_or_else(|| Status::invalid_argument("missing plan payload"))?;

        let result = match plan {
            Plan::Sql(sql) => self
                .engine
                .execute_sql(&sql.query)
                .await
                .map_err(internal_status)?,
            Plan::Projection(projection) => self
                .engine
                .execute_projection(projection.values, projection.add_scalar)
                .await
                .map_err(internal_status)?,
        };

        Ok(Response::new(ExecutePlanResponse {
            session_id: req.session_id,
            json_rows: result.json_rows,
            row_count: result.row_count as u64,
        }))
    }
}

fn internal_status(err: anyhow::Error) -> Status {
    Status::internal(err.to_string())
}
