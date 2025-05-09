use anyhow::Result;
use metrics_exporter_prometheus::PrometheusBuilder;
use std::time::Duration;

pub struct Metrics;

impl Metrics {
    pub fn new() -> Self {
        Self
    }

    pub fn record_request(&self) {
        metrics::counter!("requests_total").increment(1);
    }

    pub fn record_processing_time(&self, duration: Duration) {
        metrics::histogram!("processing_time_seconds").record(duration.as_secs_f64());
    }
}

pub fn setup_metrics() -> Result<()> {
    PrometheusBuilder::new()
        .add_global_label("service", "audio_agent")
        .install()?;
    Ok(())
}
