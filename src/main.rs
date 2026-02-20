mod engine;
mod server;

pub mod spark {
    pub mod connect {
        tonic::include_proto!("spark.connect");
    }
}

use engine::DistributedPolarsEngine;
use server::SparkConnectPolarsServer;
use spark::connect::spark_connect_service_server::SparkConnectServiceServer;
use tonic::transport::Server;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    let addr = "0.0.0.0:15002".parse()?;
    let service = SparkConnectPolarsServer::new(DistributedPolarsEngine);

    info!(%addr, "starting Spark Connect (Polars-backed) server");

    Server::builder()
        .add_service(SparkConnectServiceServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
