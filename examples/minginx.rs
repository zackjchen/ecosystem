use std::sync::Arc;

use tokio::net::{TcpListener, TcpStream};
use tracing::{info, level_filters::LevelFilter, warn};
use tracing_subscriber::{
    fmt::Layer, layer::SubscriberExt as _, util::SubscriberInitExt as _, Layer as _,
};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Config {
    listen: String,
    upstream: String,
    // workers: u8,
    // timeout: u64
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let layer = Layer::new().with_filter(LevelFilter::INFO);

    tracing_subscriber::registry().with(layer).init();

    let config = Arc::new(resolve_config());
    info!("Upstream: {}", config.upstream);
    info!("Listen: {}", config.listen);

    let listener = TcpListener::bind(&config.listen).await?;
    loop {
        let (client, addr) = listener.accept().await?;
        let config = Arc::clone(&config);
        info!("Accept connection from: {}", addr);
        tokio::spawn(async move {
            let upstream = TcpStream::connect(&config.upstream).await?;

            // proxy
            if let Err(e) = proxy(client, upstream).await {
                warn!("Error proxying: {:?}", e)
            }

            Ok::<(), anyhow::Error>(())
        });
    }
}

// 假设从文件读取config
fn resolve_config() -> Config {
    Config {
        listen: "0.0.0.0:8081".into(),
        upstream: "0.0.0.0:8080".into(),
    }
}

async fn proxy(mut client: TcpStream, mut upstream: TcpStream) -> anyhow::Result<()> {
    let (mut client_reader, mut client_writer) = client.split();
    let (mut upstream_reader, mut upstream_writer) = upstream.split();
    let client_to_upstream = tokio::io::copy(&mut client_reader, &mut upstream_writer);
    let upstream_to_client = tokio::io::copy(&mut upstream_reader, &mut client_writer);
    tokio::try_join!(client_to_upstream, upstream_to_client)?;
    Ok(())
}
