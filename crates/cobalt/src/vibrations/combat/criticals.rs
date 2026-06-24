use engage::combat::character::{Character, ICharacterMethods};
use engage::unity_engine::animationevent::{AnimationEvent, IAnimationEventMethods};
use unity::{Cast, OptionalMethod};

use crate::vibe_log;
use crate::vibrationevents::get_vibration_event;
use crate::vibrations::queue_handlers::{run_vibration_event, run_vibration_event_by_name};
use crate::vibrations::util::{do_vibrate, get_vibration_judgement, VibrationJudgement};

//Combat.CharacterSound$$<MyStart>b__21_0	71025f1c20	void Combat.CharacterSound$$<MyStart>b__21_0(Combat_CharacterSound_o * __this, Combat_Character_o * c, MethodInfo * method)	48
/// This hook is used for sound events that are synced with animations.
/// We are specifically using it for critical charge events, which is a special animation and sound effect that plays when a character is about to do a critical attack.
#[skyline::hook(offset = 0x25f1c20)]
pub fn my_start(this: *const u8, character: Character, method_info: OptionalMethod) {
    call_original!(this, character, method_info);
    do_vibrate(|| handle_my_start(character));
}

pub fn handle_my_start(character: Character) {
    let event: AnimationEvent = character.get_playing_event();
    if event.is_null() {
        return;
    }
    let raw_name = event.get_string_parameter();
    if raw_name.is_null() {
        return;
    }

    let event_name = raw_name.to_rust_string();
    vibe_log!("{}: my start event.", event_name);

    let cobalt_name = format!("Cobalt_{event_name}");
    if let Some(ref custom) = get_vibration_event(cobalt_name.as_str()) {
        vibe_log!("{}: Running custom vibration event at my_start.", cobalt_name);
        run_vibration_event(custom);
        return;
    }

    let is_critical_charge = event_name.starts_with("SE_Critical") && event_name.ends_with("Charge");
    vibe_log!("{}: is_critical_charge: {}", event_name, is_critical_charge);
    if !is_critical_charge {
        return;
    }

    if let VibrationJudgement::ShouldVibrate(_) = get_vibration_judgement(character) {
        run_vibration_event_by_name("Cobalt_Critical_Charge");
    }
}
