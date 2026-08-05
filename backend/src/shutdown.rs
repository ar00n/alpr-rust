use tokio::sync::broadcast;

pub async fn shutdown_signal(shutdown_tx: broadcast::Sender<()>) {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("🛑 Received Ctrl+C (SIGINT)");
        },
        _ = terminate => {
            tracing::info!("🛑 Received SIGTERM");
        },
    }

    tracing::info!("Shutting down application gracefully...");
    let _ = shutdown_tx.send(()); 
}