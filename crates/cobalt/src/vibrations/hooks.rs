use unity::prelude::*;

pub static mut IS_REWINDING: bool = false;

// App.RewindSequence$$Start	7102414b90	void App.RewindSequence$$Start(App_RewindSequence_o * __this, MethodInfo * method)	8
/// Prevent weird vibrations from playing due to terrain effects which trigger a sound effect when rewinded to.
#[skyline::hook(offset = 0x2414b90)]
pub fn rewind_sequence_start(this: *const u8, method_info: OptionalMethod) {
    call_original!(this, method_info);
    unsafe {
        IS_REWINDING = true;
    }
}

// App.RewindSequence$$CancelRewind	7102415450	void App.RewindSequence$$CancelRewind(App_RewindSequence_o * __this, MethodInfo * method)	108
/// Resume vibrations if rewinding is cancelled.
#[skyline::hook(offset = 0x2415450)]
pub fn rewind_sequence_cancel_rewind(this: *const u8, method_info: OptionalMethod) {
    call_original!(this, method_info);
    unsafe {
        IS_REWINDING = false;
    }
}

// App.RewindSequence$$ExecuteRewind	7102414de0	void App.RewindSequence$$ExecuteRewind(App_RewindSequence_o * __this, MethodInfo * method)	952
/// Resume vibrations if rewinding is exectued.
#[skyline::hook(offset = 0x2414de0)]
pub fn rewind_sequence_execute_rewind(this: *const u8, method_info: OptionalMethod) {
    call_original!(this, method_info);
    unsafe {
        IS_REWINDING = false;
    }
}
