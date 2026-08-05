pub async fn trigger_api(plate: &str) {
    tracing::debug!("[GATE ACTUATION] Triggered open command for plate: {}", plate);
    // e.g., reqwest::Client::new().post("http://gate-controller.local/open").send().await
}