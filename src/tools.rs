use std::time::Duration;

pub async fn wait(duration: Duration) {
    smol::Timer::after(duration).await;
}
