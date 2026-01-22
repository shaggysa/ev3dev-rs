use std::time::Duration;
use tokio::time::sleep;

pub async fn wait(duration: Duration) {
    sleep(duration).await;
}
