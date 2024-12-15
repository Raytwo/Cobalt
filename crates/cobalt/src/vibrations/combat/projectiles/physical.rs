use crate::vibrations::queue_handlers::run_vibration_event_by_name;
use crate::vibrations::util::{do_vibrate, get_vibration_judgement, ConnectionType, VibrationJudgement};
use engage::combat::{charactersound_get_cp, CharacterSound};
use unity::prelude::*;

// Combat.CharacterSound$$Shoot	71025f1470	void Combat.CharacterSound$$Shoot(Combat_CharacterSound_o * __this, MethodInfo * method)	244
#[unity::hook("Combat", "CharacterSound", "Shoot")]
pub fn character_sound_shoot(this: &CharacterSound, method_info: OptionalMethod) {
    call_original!(this, method_info);
    do_vibrate(|| {
        handle_character_sound_shoot(this, method_info);
    });
}

/// Play vibrations when a character shoots arrows, throws knives, axes, etc.
pub fn handle_character_sound_shoot(this: &CharacterSound, method_info: OptionalMethod) {
    unsafe {
        let character = charactersound_get_cp(this, method_info);
        
        match get_vibration_judgement(character) {
            VibrationJudgement::NoVibration => (),
            VibrationJudgement::ShouldVibrate(connection_type) => match connection_type {
                ConnectionType::Normal => {
                    run_vibration_event_by_name("Cobalt_Projectile_Physical_Normal");
                },
                ConnectionType::Critical => {
                    run_vibration_event_by_name("Cobalt_Projectile_Physical_Critical");
                },
            },
        }
    }
}
