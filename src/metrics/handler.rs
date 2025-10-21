use {
    crate::config::MetricsConfig,
    prometheus::{
        Counter, CounterVec, Encoder, Gauge, GaugeVec, Histogram, Registry, TextEncoder,
        proto::MetricFamily,
    },
    reqwest::Client,
    std::time::{Duration, Instant},
    tokio::{sync::broadcast::Receiver, time::interval},
    tracing::{debug, error, info},
};

lazy_static::lazy_static! {
    pub static ref REGISTRY: Registry = Registry::new();
}

const PKG_NAME: &str = env!("CARGO_PKG_NAME");
const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

pub async fn start_metrics_pusher(config: MetricsConfig, mut shutdown_receiver: Receiver<()>) {
    let client = Client::new(); // Use just one client for performance
    let mut interval = interval(Duration::from_secs(config.push_interval_seconds));

    let hostname = hostname::get()
        .ok()
        .and_then(|name| name.into_string().ok());

    let job_name = format!("quasar-{}", hostname.unwrap_or_default());
    let instance = format!("{}/{}", PKG_NAME, PKG_VERSION);

    let user_agent = format!("{}/{}", PKG_NAME, PKG_VERSION);

    info!(
        "Metrics pusher initialized for {}. Interval {}s.",
        config.remote_write_url, config.push_interval_seconds
    );

    loop {
        tokio::select! {
            _ = interval.tick() => {
                match push_metrics(client.clone(), config.remote_write_url.clone(), job_name.clone(), instance.clone(), user_agent.clone()).await {
                    Ok(_) => debug!("Metrics pushed successfully"),
                    Err(e) => error!("Error pushing metrics: {}", e),
                }
            }
            _ = shutdown_receiver.recv() => {
                info!("Shutting down metrics pusher...");
                break;
            }
        }
    }
}

async fn push_metrics(
    client: Client,
    remote_url: String,
    job_name: String,
    instance: String,
    user_agent: String,
) -> Result<(), String> {
    // Collect all metrics from global registry
    let metric_families = REGISTRY.gather();

    if metric_families.is_empty() {
        // No metrics to push
        return Ok(());
    }

    let start_time = Instant::now();

    // Send metrics as plain text via HTTP POST
    let response = client
        .post(&remote_url)
        .header("Content-Type", "text/plain; version=0.0.4; charset=utf-8")
        .header("User-Agent", user_agent)
        .body(inject_job_label(
            &metrics_to_text(metric_families)?,
            &job_name,
            &instance,
        ))
        .send()
        .await;

    match response {
        Ok(resp) => {
            if resp.status().is_success() {
                debug!(
                    "Successfully sent metrics in {}us.",
                    start_time.elapsed().as_micros()
                );
            } else {
                let status = resp.status();
                let body = resp
                    .text()
                    .await
                    .unwrap_or_else(|_| "Failed to read response body".to_string());
                error!("Failed to send metrics. Status: {}. Body: {}", status, body);
                return Err(format!("HTTP error: {} - {}", status, body));
            }
        }
        Err(e) => {
            error!("Error on HTTP request: {}", e);
            return Err(format!("HTTP request failed: {}", e));
        }
    }

    Ok(())
}

fn metrics_to_text(metric_families: Vec<MetricFamily>) -> Result<String, String> {
    // Convert metrics to plain text format
    let encoder = TextEncoder::new();
    let mut buffer = Vec::new();

    if let Err(e) = encoder.encode(&metric_families, &mut buffer) {
        return Err(format!("Failed to encode metrics: {}", e));
    }

    String::from_utf8(buffer).map_err(|e| format!("Failed to convert metrics to string: {}", e))
}

pub fn inject_job_label(metrics: &str, job: &str, instance: &str) -> String {
    metrics
        .lines()
        .map(|line| {
            if line.starts_with("#") {
                line.to_string()
            } else if let Some(pos) = line.find('{') {
                let (metric_name, rest) = line.split_at(pos + 1);
                format!(
                    r#"{}job="{}", instance="{}", {rest}"#,
                    metric_name, job, instance
                )
            } else if let Some(pos) = line.find(' ') {
                let (metric_name, value) = line.split_at(pos);
                format!(r#"{metric_name}{{job="{job}", instance="{instance}"}}{value}"#)
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn counter(name: &str, help: &str) -> Counter {
    let counter = Counter::new(name, help).unwrap();
    if let Err(e) = REGISTRY.register(Box::new(counter.clone())) {
        if !e.to_string().contains("already registered") {
            error!("Failed to register counter {}: {}", name, e);
        }
    }
    counter
}

pub fn counter_vec(name: &str, help: &str, labels: &[&str]) -> CounterVec {
    let opts = prometheus::opts!(name, help).const_labels(
        labels
            .iter()
            .map(|&l| (l.to_string(), "".to_string()))
            .collect(),
    );
    let counter_vec = CounterVec::new(opts, labels).unwrap();
    if let Err(e) = REGISTRY.register(Box::new(counter_vec.clone())) {
        if !e.to_string().contains("already registered") {
            error!("Failed to register counter vector {}: {}", name, e);
        }
    }
    counter_vec
}

pub fn gauge(name: &str, help: &str) -> Gauge {
    let gauge = Gauge::new(name, help).unwrap();
    if let Err(e) = REGISTRY.register(Box::new(gauge.clone())) {
        if !e.to_string().contains("already registered") {
            error!("Failed to register gauge {}: {}", name, e);
        }
    }
    gauge
}

pub fn gauge_vec(name: &str, help: &str, labels: &[&str]) -> GaugeVec {
    let opts = prometheus::opts!(name, help);
    let gauge_vec = GaugeVec::new(opts, labels).unwrap();
    if let Err(e) = REGISTRY.register(Box::new(gauge_vec.clone())) {
        if !e.to_string().contains("already registered") {
            error!("Failed to register gauge vector {}: {}", name, e);
        }
    }
    gauge_vec
}

// Microsecond precision histogram for low latency operations (e.g., pool lookups, analysis)
// Buckets: 1us to 10ms
pub fn histogram_microseconds(name: &str, help: &str) -> Histogram {
    let buckets = vec![
        0.000001, // 1us
        0.000005, // 5us
        0.00001,  // 10us
        0.00005,  // 50us
        0.0001,   // 100us
        0.0005,   // 500us
        0.001,    // 1ms
        0.002,    // 2ms
        0.005,    // 5ms
        0.01,     // 10ms
    ];

    let opts = prometheus::histogram_opts!(name, help, buckets);
    let histogram = Histogram::with_opts(opts).unwrap();

    if let Err(e) = REGISTRY.register(Box::new(histogram.clone())) {
        if !e.to_string().contains("already registered") {
            error!("Failed to register histogram {}: {}", name, e);
        }
    }

    histogram
}

// Millisecond precision histogram for transaction building and processing
// Buckets: 100us to 100ms
pub fn histogram_milliseconds(name: &str, help: &str) -> Histogram {
    let buckets = vec![
        0.0001, // 100us
        0.0005, // 500us
        0.001,  // 1ms
        0.005,  // 5ms
        0.01,   // 10ms
        0.025,  // 25ms
        0.05,   // 50ms
        0.075,  // 75ms
        0.1,    // 100ms
    ];

    let opts = prometheus::histogram_opts!(name, help, buckets);
    let histogram = Histogram::with_opts(opts).unwrap();

    if let Err(e) = REGISTRY.register(Box::new(histogram.clone())) {
        if !e.to_string().contains("already registered") {
            error!("Failed to register histogram {}: {}", name, e);
        }
    }

    histogram
}
