use std::time::Duration;

use axum::{routing::get, Router};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{runtime, trace::Tracer};
use tokio::{
    net::TcpListener,
    time::{sleep, Instant},
};
use tracing::{info, instrument, level_filters::LevelFilter, warn};
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    Layer,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 输出到控制台
    let console = fmt::Layer::new()
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .pretty()
        .with_filter(LevelFilter::INFO);
    // 输出到文件
    let file_appender = tracing_appender::rolling::daily("temp/logs", "ecosystem.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    let file = fmt::Layer::new()
        // .with_span_events(FmtSpan::CLOSE)
        .with_writer(non_blocking)
        .pretty()
        .with_filter(LevelFilter::INFO);

    let tracing = init_tracer()?;
    let opentelemetry = tracing_opentelemetry::layer().with_tracer(tracing);

    tracing_subscriber::registry()
        .with(console)
        .with(file)
        .with(opentelemetry)
        .init();

    let addr = "0.0.0.0:8080";
    let app = Router::new()
        .route("/", get(index_handler))
        .route("/loog", get(loog_task));

    info!("Listening on http://{}", addr);
    let listener = TcpListener::bind(addr).await?;

    axum::serve(listener, app).await?;

    Ok(())
}

#[instrument]
async fn index_handler() -> &'static str {
    info!("access /");
    // "Hello, World!"
    let ret = loog_task().await;
    info!(http.status = 200, "response" = %ret);
    ret
}

#[instrument]
async fn loog_task() -> &'static str {
    // info!("start long task");
    let start = Instant::now();
    sleep(Duration::from_secs(1)).await;
    // info!("end long task");
    let elapsed = start.elapsed().as_millis() as f64;
    warn!(app.task_duration = elapsed, "long task complete");
    "long task"
}

fn init_tracer() -> anyhow::Result<Tracer> {
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint("http:://localhost:4317"),
        )
        .install_batch(runtime::Tokio)?;

    Ok(tracer)
}
