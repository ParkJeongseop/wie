use core::sync::atomic::{AtomicUsize, Ordering};

use jvm::{JavaError, Jvm};
use wie_util::WieError;

/// How many wie-side failures are being raised as Java exceptions right now.
static RAISING: AtomicUsize = AtomicUsize::new(0);

/// Raises a wie-side failure as a Java exception of `class_name`. Building the exception allocates
/// and may define classes; when that fails as well (heap exhausted, stub space exhausted, ...)
/// nothing can be raised, and retrying would recurse until the host stack overflows. A failure
/// while another one is being raised therefore aborts with both messages instead.
pub(crate) async fn raise(jvm: &Jvm, class_name: &str, message: &str) -> JavaError {
    if RAISING.fetch_add(1, Ordering::SeqCst) > 0 {
        RAISING.fetch_sub(1, Ordering::SeqCst);
        panic!("Fatal error while raising {class_name}: {message}");
    }

    let exception = jvm.exception(class_name, message).await;
    RAISING.fetch_sub(1, Ordering::SeqCst);

    exception
}

/// Raises an instantiation failure: heap exhaustion becomes `java/lang/OutOfMemoryError`, anything
/// else a `net/wie/WieError` naming the cause.
pub(crate) async fn raise_instantiation_failure(jvm: &Jvm, what: &str, error: WieError) -> JavaError {
    match error {
        WieError::AllocationFailure => raise(jvm, "java/lang/OutOfMemoryError", what).await,
        error => raise(jvm, "net/wie/WieError", &alloc::format!("Failed to instantiate {what}: {error}")).await,
    }
}
