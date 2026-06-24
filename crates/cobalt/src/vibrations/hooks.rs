use std::sync::atomic::{AtomicBool, Ordering};

use unity::OptionalMethod;

pub static IS_REWINDING: AtomicBool = AtomicBool::new(false);

#[inline]
pub fn is_rewinding() -> bool {
    IS_REWINDING.load(Ordering::Relaxed)
}

#[unity::hook("App", "RewindSequence", "Start")]
pub fn rewind_sequence_start(this: *const u8, method_info: OptionalMethod) {
    call_original!(this, method_info);
    IS_REWINDING.store(true, Ordering::Relaxed);
}

// App.RewindSequence$$CancelRewind	7102415450	void App.RewindSequence$$CancelRewind(App_RewindSequence_o * __this, MethodInfo * method)	108
/// Resume vibrations if rewinding is cancelled.
#[unity::hook("App", "RewindSequence", "CancelRewind")]
pub fn rewind_sequence_cancel_rewind(this: *const u8, method_info: OptionalMethod) {
    call_original!(this, method_info);
    IS_REWINDING.store(false, Ordering::Relaxed);
}

// App.RewindSequence$$ExecuteRewind	7102414de0	void App.RewindSequence$$ExecuteRewind(App_RewindSequence_o * __this, MethodInfo * method)	952
/// Resume vibrations if rewinding is exectued.
#[unity::hook("App", "RewindSequence", "ExecuteRewind")]
pub fn rewind_sequence_execute_rewind(this: *const u8, method_info: OptionalMethod) {
    call_original!(this, method_info);
    IS_REWINDING.store(false, Ordering::Relaxed);
}
