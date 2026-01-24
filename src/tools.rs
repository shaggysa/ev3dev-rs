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
/// use ev3dev_rs::join;
/// join!(drive.straight(100), attachment_motor.run_until_stalled(-45))?;
/// ```
#[macro_export]
macro_rules! join {
    ($($fut:expr),+ $(,)?) => {
         tokio::try_join!($($fut),+)
    };
}

/// A racing multitasker
///
/// # Examples
/// ```
/// use ev3dev_rs::select;
/// select!(drive.straight(100), attachment_motor.run_until_stalled(-45))?;
/// ```
#[macro_export]
macro_rules! select {
    ($($fut:expr),+ $(,)?) => {
        ev3dev_rs::Race::race(($($fut),+)).await
    };
}
