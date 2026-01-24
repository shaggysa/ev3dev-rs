use std::time::Duration;
use tokio::time::sleep;

/// A pybricks-like async wait function.
pub async fn wait(duration: Duration) {
    sleep(duration).await;
}
/// A non-racing multitasker.
///
/// # Examples
/// ```
/// use ev3dev_rs::tools::join;
/// join!(drive.straight(100), attachment_motor.run_until_stalled(-45))?;
/// ```
pub macro join($($fut:expr),+ $(,)?) {
    tokio::try_join!($($fut),+)
}

/// A racing multitasker
///
/// # Examples
/// ```
/// use ev3dev_rs::tools::select;
/// select!(drive.straight(100), attachment_motor.run_until_stalled(-45)).await?;
/// ```
pub macro select($($fut:expr),+ $(,)?) {
async {
    use ev3dev_rs::Race;
    ($($fut),+).race().await
    }
}
