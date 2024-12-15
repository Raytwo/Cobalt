use engage::{
    gamevariable::GameVariableManager,
    vibrationmanager::{vibrate as actually_vibrate, FREQ_HIGH, FREQ_LOW},
};
use unity::prelude::*;

use crate::config::ringpolishrumble::ENABLE_POLISH_VIBRATION_KEY;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum Strength {
    Strong,
    Weak,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum HitResult {
    NotHit,
    Near,
    Hit,
}

// App.RingCleaningSequence$$CleanRing	710241ee00	void App.RingCleaningSequence$$CleanRing(App_RingCleaningSequence_o * __this, int32_t strength, int32_t hitResult, MethodInfo * method)	212
/// This gets called when we actually clean a spot on the ring... normal rubs don't trigger this.
#[unity::hook("App", "RingCleaningSequence", "CleanRing")]
pub fn clean_ring(this: *const u8, strength: Strength, hit_result: HitResult, method_info: OptionalMethod) {
    call_original!(this, strength, hit_result, method_info);
    println!("CleanRing: {:?} {:?}", strength, hit_result);
    let (time, amplitude_magnitude, amp_low, amp_high) = match (strength, hit_result) {
        (Strength::Strong, HitResult::Hit) => (0.35, 1.0, 0.4, 0.8),
        (Strength::Strong, HitResult::Near) => (0.25, 0.75, 0.3, 0.7),
        (Strength::Weak, HitResult::Hit) => (0.15, 0.35, 1.0, 1.0),
        (Strength::Weak, HitResult::Near) => (0.15, 0.25, 0.3, 0.7),
        (Strength::Strong, HitResult::NotHit) => (0.35, 0.25, 0.2, 0.5),
        (Strength::Weak, HitResult::NotHit) => (0.25, 0.25, 0.2, 0.5),
    };
    vibrate(time, amplitude_magnitude, amp_low, amp_high, FREQ_LOW, FREQ_HIGH);
}
// App.RingCleaning.EffectController$$PlayRubEffect	71022d7f80	UnityEngine_GameObject_o * App.RingCleaning.EffectController$$PlayRubEffect(MethodInfo * method)	272
// UnityEngine_GameObject_o * App.RingCleaning.EffectController$$PlayRubEffect(MethodInfo *method)
/// This gets called when holding down R and rotating the stick.
#[skyline::hook(offset = 0x22d7f80)]
pub fn play_rub_effect(method_info: OptionalMethod) -> *const u8 {
    let result = call_original!(method_info);
    println!("PlayRubEffect");
    vibrate(0.15, 0.15, 0.05, 0.15, FREQ_LOW, FREQ_HIGH);
    result
}

// App.RingCleaningClothAnimationEvent$$CleaningStartEvent	71024172e0	void App.RingCleaningClothAnimationEvent$$CleaningStartEvent(App_RingCleaningClothAnimationEvent_o * __this, MethodInfo * method)	20

/// This gets called when just pressing A to clean.
#[unity::hook("App", "RingCleaningClothAnimationEvent", "CleaningStartEvent")]
pub fn cleaning_start_event(this: *const u8, method_info: OptionalMethod) {
    call_original!(this, method_info);
    println!("CleaningStartEvent");
    vibrate(0.12, 0.10, 0.05, 0.10, FREQ_LOW, FREQ_HIGH);
}

fn vibrate(time: f32, amplitude_magnitude: f32, amp_low: f32, amp_high: f32, freq_low: f32, freq_high: f32) {
    let vibration_enabled = GameVariableManager::get_bool(ENABLE_POLISH_VIBRATION_KEY);
    match vibration_enabled {
        true => actually_vibrate(time, amplitude_magnitude, amp_low, amp_high, freq_low, freq_high),
        false => (),
    }
}
