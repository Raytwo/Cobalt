#![allow(unused)]

extern "C" {
    pub fn sky_Hook(hook: u64, callback: u64, do_trampoline: bool) -> u64;
    pub fn sky_InlineHook(hook: u64, callback: u64);
}
