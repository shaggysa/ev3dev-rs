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
/// select!(drive.straight(100), attachment_motor.run_until_stalled(-45))?;
/// ```
pub macro select($($fut:expr),+ $(,)?) {{
    $crate::__select_internal!([], $($fut),+)
}}

#[macro_export]
#[doc(hidden)]
macro_rules! __select_internal {
    // Final expansion: no more futures
    ([$($arms:tt)*],) => {
        tokio::select! { $($arms)* }
    };

    // Recursive case: grab head future
    ([$($arms:tt)*], $head:expr $(, $tail:expr)*) => {
        // bind the future in the outer scope
        std::pin::pin!(__fut = $head);

        // recursively accumulate the arms
        $crate::__select_internal!(
            [
                $($arms)*
                res = &mut __fut => res?,
            ],
            $($tail),*
        )
    };
}
