use std::time::Duration;
use tokio::time::sleep;

/// A pybricks-like async wait function.
pub async fn wait(duration: Duration) {
    sleep(duration).await;
}
