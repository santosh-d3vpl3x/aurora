mod distributed;
mod server;

pub mod spark {
    pub mod connect {
        tonic::include_proto!("spark.connect");
    }
}

use anyhow::Result;
use server::SparkConnectServer;
use tonic::transport::Server;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    let addr = std::env::var("SPARK_CONNECT_ADDR").unwrap_or_else(|_| "0.0.0.0:15002".to_string());
    info!(%addr, "starting Spark Connect server over distributed Polars");

    Server::builder()
        .add_service(SparkConnectServer::default().into_service())
        .serve(addr.parse()?)
        .await?;

    Ok(())
}
