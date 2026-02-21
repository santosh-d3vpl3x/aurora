use crate::distributed::DistributedPolars;
use crate::spark::connect::analyze_plan_response::Result as AnalyzeResult;
use crate::spark::connect::execute_plan_request::Command;
use crate::spark::connect::spark_connect_service_server::SparkConnectService;
use crate::spark::connect::{
    AddArtifactsRequest, AddArtifactsResponse, AnalyzePlanRequest, AnalyzePlanResponse, ArrowBatch,
    ArtifactStatusesRequest, ArtifactStatusesResponse, ConfigRequest, ConfigResponse, Error,
    ExecuteMetrics, ExecutePlanRequest, ExecutePlanResponse, Field, InterruptRequest,
    InterruptResponse, JsonRows, KeyValue, ReattachExecuteRequest, ReleaseExecuteRequest,
    ReleaseExecuteResponse, ResultComplete, Schema,
};
use base64::Engine;
use futures::Stream;
use polars::prelude::{AnyValue, DataFrame, SchemaExt};
use serde_json::Map;
use std::collections::{HashMap, HashSet};
use std::pin::Pin;
use std::process::Command as ProcessCommand;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tonic::{Request, Response as TonicResponse, Status};

#[derive(Debug, Default)]
struct ServerState {
    artifacts: HashMap<String, Vec<u8>>,
    operation_frames: HashMap<String, Vec<ExecutePlanResponse>>,
    interrupted: HashSet<String>,
    operation_counter: u64,
}

#[derive(Debug, Clone, Default)]
pub struct SparkConnectServer {
    executor: DistributedPolars,
    state: Arc<Mutex<ServerState>>,
}

#[tonic::async_trait]
impl SparkConnectService for SparkConnectServer {
    type ExecutePlanStream =
        Pin<Box<dyn Stream<Item = Result<ExecutePlanResponse, Status>> + Send>>;
    type ReattachExecuteStream =
        Pin<Box<dyn Stream<Item = Result<ExecutePlanResponse, Status>> + Send>>;

    async fn execute_plan(
        &self,
        request: Request<ExecutePlanRequest>,
    ) -> Result<TonicResponse<Self::ExecutePlanStream>, Status> {
        let req = request.into_inner();
        let session_id = req.session_id;
        let operation_id = self.resolve_operation_id(req.operation_id);
        let start = Instant::now();

        let interrupted = {
            let state = self
                .state
                .lock()
                .map_err(|_| Status::internal("state lock poisoned"))?;
            state.interrupted.contains(&operation_id)
        };

        let frames = if interrupted {
            vec![error_frame(
                &session_id,
                &operation_id,
                "1",
                "operation interrupted",
            )]
        } else if let Some(plan) = req.plan {
            match evaluate_spark_plan(&session_id, &operation_id, &plan.root) {
                Ok((arrow_data, row_count, _fields)) => vec![
                    ExecutePlanResponse {
                        session_id: session_id.clone(),
                        operation_id: operation_id.clone(),
                        response_id: "1".to_string(),
                        arrow_batch: Some(ArrowBatch {
                            row_count: row_count as i64,
                            data: arrow_data,
                            start_offset: 0,
                            chunk_index: 0,
                            num_chunks_in_batch: 1,
                        }),
                        ..Default::default()
                    },
                    ExecutePlanResponse {
                        session_id: session_id.clone(),
                        operation_id: operation_id.clone(),
                        response_id: "2".to_string(),
                        metrics: Some(ExecuteMetrics {
                            elapsed_ms: start.elapsed().as_millis() as u64,
                            row_count: row_count as u64,
                        }),
                        ..Default::default()
                    },
                    ExecutePlanResponse {
                        session_id: session_id.clone(),
                        operation_id: operation_id.clone(),
                        response_id: "3".to_string(),
                        result_complete: Some(ResultComplete {}),
                        ..Default::default()
                    },
                ],
                Err(e) => vec![error_frame(&session_id, &operation_id, "1", &e)],
            }
        } else {
            match execute_dataframe(&self.executor, req.command) {
                Ok(df) => response_frames_for_df(
                    &session_id,
                    &operation_id,
                    &df,
                    start.elapsed().as_millis() as u64,
                ),
                Err(message) => vec![error_frame(&session_id, &operation_id, "1", &message)],
            }
        };

        {
            let mut state = self
                .state
                .lock()
                .map_err(|_| Status::internal("state lock poisoned"))?;
            state
                .operation_frames
                .insert(operation_id.clone(), frames.clone());
        }

        let stream = async_stream::try_stream! { for frame in frames { yield frame; } };
        Ok(TonicResponse::new(
            Box::pin(stream) as Self::ExecutePlanStream
        ))
    }

    async fn analyze_plan(
        &self,
        request: Request<AnalyzePlanRequest>,
    ) -> Result<TonicResponse<AnalyzePlanResponse>, Status> {
        let req = request.into_inner();
        let Some(plan_req) = req.plan else {
            return Ok(TonicResponse::new(AnalyzePlanResponse {
                session_id: req.session_id,
                result: Some(AnalyzeResult::Error(Error {
                    message: "missing plan payload".to_string(),
                })),
            }));
        };

        let result = if let Some(plan) = plan_req.plan {
            match evaluate_spark_plan(&req.session_id, "analyze", &plan.root) {
                Ok((_, _, fields)) => AnalyzeResult::Schema(Schema { fields }),
                Err(message) => AnalyzeResult::Error(Error { message }),
            }
        } else {
            match execute_dataframe(&self.executor, plan_req.command) {
                Ok(df) => AnalyzeResult::Schema(Schema {
                    fields: schema_fields(&df),
                }),
                Err(message) => AnalyzeResult::Error(Error { message }),
            }
        };

        Ok(TonicResponse::new(AnalyzePlanResponse {
            session_id: req.session_id,
            result: Some(result),
        }))
    }

    async fn config(
        &self,
        request: Request<ConfigRequest>,
    ) -> Result<TonicResponse<ConfigResponse>, Status> {
        let req = request.into_inner();
        let (artifact_count, op_count) = {
            let state = self
                .state
                .lock()
                .map_err(|_| Status::internal("state lock poisoned"))?;
            (state.artifacts.len(), state.operation_frames.len())
        };
        Ok(TonicResponse::new(ConfigResponse {
            session_id: req.session_id,
            pairs: vec![
                KeyValue {
                    key: "spark.connect.service".to_string(),
                    value: "aurora".to_string(),
                },
                KeyValue {
                    key: "aurora.artifact_count".to_string(),
                    value: artifact_count.to_string(),
                },
                KeyValue {
                    key: "aurora.cached_operation_count".to_string(),
                    value: op_count.to_string(),
                },
            ],
            warnings: vec![],
        }))
    }

    async fn add_artifacts(
        &self,
        request: Request<tonic::Streaming<AddArtifactsRequest>>,
    ) -> Result<TonicResponse<AddArtifactsResponse>, Status> {
        let mut stream = request.into_inner();
        let mut summaries = Vec::new();
        while let Some(msg) = stream.message().await? {
            match msg.artifact {
                Some(crate::spark::connect::add_artifacts_request::Artifact::Batch(batch)) => {
                    for artifact in batch.artifacts {
                        let name = artifact.name.clone();
                        let size = artifact.data.len();
                        self.state
                            .lock()
                            .map_err(|_| Status::internal("state lock poisoned"))?
                            .artifacts
                            .insert(name.clone(), artifact.data);
                        summaries.push(
                            crate::spark::connect::add_artifacts_response::ArtifactSummary {
                                name,
                                success: true,
                                message: format!("stored {size} bytes"),
                            },
                        );
                    }
                }
                _ => {}
            }
        }
        Ok(TonicResponse::new(AddArtifactsResponse {
            artifacts: summaries,
        }))
    }

    async fn artifact_status(
        &self,
        request: Request<ArtifactStatusesRequest>,
    ) -> Result<TonicResponse<ArtifactStatusesResponse>, Status> {
        let req = request.into_inner();
        let state = self
            .state
            .lock()
            .map_err(|_| Status::internal("state lock poisoned"))?;
        let statuses = req
            .names
            .into_iter()
            .map(|name| {
                (
                    name.clone(),
                    crate::spark::connect::artifact_statuses_response::ArtifactStatus {
                        exists: state.artifacts.contains_key(&name),
                    },
                )
            })
            .collect();
        Ok(TonicResponse::new(ArtifactStatusesResponse { statuses }))
    }

    async fn interrupt(
        &self,
        request: Request<InterruptRequest>,
    ) -> Result<TonicResponse<InterruptResponse>, Status> {
        let req = request.into_inner();
        let mut state = self
            .state
            .lock()
            .map_err(|_| Status::internal("state lock poisoned"))?;
        let interrupted_ids = if req.operation_id.is_empty() {
            let ids: Vec<String> = state.operation_frames.keys().cloned().collect();
            for id in &ids {
                state.interrupted.insert(id.clone());
            }
            ids
        } else {
            state.interrupted.insert(req.operation_id.clone());
            vec![req.operation_id]
        };
        Ok(TonicResponse::new(InterruptResponse {
            session_id: req.session_id,
            interrupted_ids,
        }))
    }

    async fn reattach_execute(
        &self,
        request: Request<ReattachExecuteRequest>,
    ) -> Result<TonicResponse<Self::ReattachExecuteStream>, Status> {
        let req = request.into_inner();
        let start_after = req.last_response_id.parse::<usize>().unwrap_or(0);
        let frames = {
            self.state
                .lock()
                .map_err(|_| Status::internal("state lock poisoned"))?
                .operation_frames
                .get(&req.operation_id)
                .cloned()
                .unwrap_or_default()
        };
        let filtered: Vec<_> = frames
            .into_iter()
            .filter(|f| f.response_id.parse::<usize>().unwrap_or(0) > start_after)
            .collect();
        let stream = async_stream::try_stream! { for frame in filtered { yield frame; } };
        Ok(TonicResponse::new(
            Box::pin(stream) as Self::ReattachExecuteStream
        ))
    }

    async fn release_execute(
        &self,
        request: Request<ReleaseExecuteRequest>,
    ) -> Result<TonicResponse<ReleaseExecuteResponse>, Status> {
        let req = request.into_inner();
        let mut state = self
            .state
            .lock()
            .map_err(|_| Status::internal("state lock poisoned"))?;
        state.operation_frames.remove(&req.operation_id);
        state.interrupted.remove(&req.operation_id);
        Ok(TonicResponse::new(ReleaseExecuteResponse {
            session_id: req.session_id,
            operation_id: req.operation_id,
        }))
    }
}

fn evaluate_spark_plan(
    session_id: &str,
    operation_id: &str,
    root: &[u8],
) -> Result<(Vec<u8>, usize, Vec<Field>), String> {
    let out = ProcessCommand::new("python3")
        .arg("scripts/plan_eval.py")
        .arg("--root-b64")
        .arg(base64::engine::general_purpose::STANDARD.encode(root))
        .output()
        .map_err(|e| format!("failed to run plan evaluator: {e}"))?;

    if !out.status.success() {
        return Err(format!(
            "plan evaluator failed: {}",
            String::from_utf8_lossy(&out.stderr)
        ));
    }

    let v: serde_json::Value = serde_json::from_slice(&out.stdout)
        .map_err(|e| format!("invalid evaluator output: {e}"))?;
    let arrow = base64::engine::general_purpose::STANDARD
        .decode(v["arrow_b64"].as_str().unwrap_or_default())
        .map_err(|e| e.to_string())?;
    let row_count = v["row_count"].as_u64().unwrap_or(0) as usize;
    let fields = v["fields"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .map(|f| Field {
            name: f["name"].as_str().unwrap_or_default().to_string(),
            dtype: f["dtype"].as_str().unwrap_or_default().to_string(),
        })
        .collect();
    let _ = (session_id, operation_id);
    Ok((arrow, row_count, fields))
}

fn error_frame(
    session_id: &str,
    operation_id: &str,
    response_id: &str,
    message: &str,
) -> ExecutePlanResponse {
    ExecutePlanResponse {
        session_id: session_id.to_string(),
        operation_id: operation_id.to_string(),
        response_id: response_id.to_string(),
        error: Some(Error {
            message: message.to_string(),
        }),
        ..Default::default()
    }
}

fn response_frames_for_df(
    session_id: &str,
    operation_id: &str,
    df: &DataFrame,
    elapsed_ms: u64,
) -> Vec<ExecutePlanResponse> {
    let arrow = Vec::new();
    vec![
        ExecutePlanResponse {
            session_id: session_id.to_string(),
            operation_id: operation_id.to_string(),
            response_id: "1".to_string(),
            arrow_batch: Some(ArrowBatch {
                row_count: df.height() as i64,
                data: arrow,
                start_offset: 0,
                chunk_index: 0,
                num_chunks_in_batch: 1,
            }),
            ..Default::default()
        },
        ExecutePlanResponse {
            session_id: session_id.to_string(),
            operation_id: operation_id.to_string(),
            response_id: "2".to_string(),
            rows: Some(JsonRows {
                rows: rows_to_json(df),
            }),
            ..Default::default()
        },
        ExecutePlanResponse {
            session_id: session_id.to_string(),
            operation_id: operation_id.to_string(),
            response_id: "3".to_string(),
            metrics: Some(ExecuteMetrics {
                elapsed_ms,
                row_count: df.height() as u64,
            }),
            ..Default::default()
        },
        ExecutePlanResponse {
            session_id: session_id.to_string(),
            operation_id: operation_id.to_string(),
            response_id: "4".to_string(),
            result_complete: Some(ResultComplete {}),
            ..Default::default()
        },
    ]
}

fn execute_dataframe(
    executor: &DistributedPolars,
    command: Option<Command>,
) -> Result<DataFrame, String> {
    match command {
        Some(Command::SqlCommand(sql)) => executor
            .execute_sql(&sql.query, &sql.table_paths, sql.partitions as usize)
            .map_err(|e| e.to_string()),
        Some(Command::ProjectionCommand(projection)) => executor
            .execute_projection(projection.values, projection.add_scalar)
            .map_err(|e| e.to_string()),
        Some(Command::RangeRepartitionCountCommand(cmd)) => executor
            .execute_range_repartition_count(
                cmd.start,
                cmd.end,
                cmd.step,
                cmd.partitions as usize,
                cmd.repartition as usize,
            )
            .map_err(|e| e.to_string()),
        None => Err("missing command payload".to_string()),
    }
}

fn schema_fields(df: &DataFrame) -> Vec<Field> {
    df.schema()
        .iter_fields()
        .map(|f| Field {
            name: f.name().to_string(),
            dtype: format!("{:?}", f.dtype()),
        })
        .collect()
}

fn rows_to_json(df: &DataFrame) -> Vec<String> {
    (0..df.height())
        .map(|idx| {
            let mut obj = Map::new();
            for col in df.get_columns() {
                let value = match col.get(idx) {
                    Ok(AnyValue::Null) => serde_json::Value::Null,
                    Ok(v) => serde_json::Value::String(v.to_string()),
                    Err(_) => serde_json::Value::Null,
                };
                obj.insert(col.name().to_string(), value);
            }
            serde_json::Value::Object(obj).to_string()
        })
        .collect()
}

impl SparkConnectServer {
    fn resolve_operation_id(&self, requested: String) -> String {
        if !requested.is_empty() {
            return requested;
        }
        let mut state = self.state.lock().expect("state lock poisoned");
        state.operation_counter += 1;
        format!("op-{}", state.operation_counter)
    }

    pub fn into_service(
        self,
    ) -> crate::spark::connect::spark_connect_service_server::SparkConnectServiceServer<Self> {
        crate::spark::connect::spark_connect_service_server::SparkConnectServiceServer::new(self)
    }
}
