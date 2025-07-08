#[cfg(feature = "dev_metrics")]
mod prom {
    use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
    static mut HANDLE: Option<PrometheusHandle> = None;

    pub fn init() {
        let builder = PrometheusBuilder::new();
        let handle = builder.install().expect("prometheus recorder install");
        unsafe { HANDLE = Some(handle); }
    }

    /// Get exposition string for scraping.
    pub fn render() -> String {
        unsafe { HANDLE.as_ref().map(|h| h.render()).unwrap_or_default() }
    }
}

#[cfg(not(feature = "dev_metrics"))]
mod prom {
    pub fn init() {}
    pub fn render() -> String { String::new() }
}

pub use prom::*; 