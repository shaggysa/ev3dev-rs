use std::time::Duration;

pub async fn wait(time: Duration) {
    smol::Timer::after(time).await;
}
