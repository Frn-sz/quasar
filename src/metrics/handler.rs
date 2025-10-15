use crate::config::VictoriaMetricsConfig;
use prometheus_reqwest_remote_write::WriteRequest;
use std::time::Duration;
use tracing::{error, info};

pub async fn start_metrics_pusher(
    config: VictoriaMetricsConfig,
    mut shutdown_receiver: tokio::sync::broadcast::Receiver<()>,
) {
    info!(
        "Initializing metrics pusher for {}",
        config.remote_write_url
    );
    let http_client = reqwest::Client::new();
    let mut interval = tokio::time::interval(Duration::from_secs(config.push_interval_seconds));
    let registry = prometheus::default_registry();
    loop {
        tokio::select! {
            _ = interval.tick() => {
                push_metrics(
                    &http_client,
                    &config.remote_write_url,
                    registry,
                ).await;
            }
            _ = shutdown_receiver.recv() => {
                info!("Shutting down metrics pusher");
                break;
            }
        }
    }
}

pub async fn push_metrics(
    client: &reqwest::Client,
    remote_write_url: &str,
    registry: &prometheus::Registry,
) {
    info!("Registry has {} metrics", registry.gather().len());
    let write_request = WriteRequest::from_metric_families(registry.gather(), None)
        .expect("Could not format write request");

    let http_request = write_request
        .build_http_request(client.clone(), remote_write_url, "your_user_agent")
        .expect("Could not build http request");

    match client.execute(http_request).await {
        Ok(r) => {
            if r.status().is_success() {
                info!("Metrics sent successfully");
            } else {
                error!(
                    "Failed to send metrics: {:?}",
                    r.text().await.expect("Could not read body from response")
                );
            }
        }
        Err(e) => {
            error!("Failed to send metrics: {:?}", e);
        }
    }
}
