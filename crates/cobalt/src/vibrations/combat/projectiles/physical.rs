use engage::combat::charactersound::{CharacterSound, ICharacterSoundMethods};
use unity::OptionalMethod;

use crate::vibrations::queue_handlers::run_vibration_event_by_name;
use crate::vibrations::util::{do_vibrate, get_vibration_judgement, ConnectionType, VibrationJudgement};

// Combat.CharacterSound$$Shoot	71025f1470	void Combat.CharacterSound$$Shoot(Combat_CharacterSound_o * __this, MethodInfo * method)	244
#[unity::hook("Combat", "CharacterSound", "Shoot")]
pub fn character_sound_shoot(this: CharacterSound, method_info: OptionalMethod) {
    call_original!(this, method_info);
    do_vibrate(|| handle_character_sound_shoot(this));
}

/// Play vibrations when a character shoots arrows, throws knives, axes, etc.
pub fn handle_character_sound_shoot(this: CharacterSound) {
    let character = this.get_cp();
    match get_vibration_judgement(character) {
        VibrationJudgement::NoVibration => (),
        VibrationJudgement::ShouldVibrate(connection) => match connection {
            ConnectionType::Normal => run_vibration_event_by_name("Cobalt_Projectile_Physical_Normal"),
            ConnectionType::Critical => run_vibration_event_by_name("Cobalt_Projectile_Physical_Critical"),
        },
    }
}
