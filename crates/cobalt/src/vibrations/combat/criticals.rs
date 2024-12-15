use crate::vibe_log;
use crate::vibrations::queue_handlers::run_vibration_event;
use crate::vibrations::util::{do_vibrate, get_vibration_judgement, VibrationJudgement};
use engage::combat::Character;

use crate::vibrations::queue_handlers::run_vibration_event_by_name;
use engage::unityengine::animationevent_get_stringparameter;
use unity::prelude::*;

use crate::vibrationevents::get_vibration_event;

//Combat.CharacterSound$$<MyStart>b__21_0	71025f1c20	void Combat.CharacterSound$$<MyStart>b__21_0(Combat_CharacterSound_o * __this, Combat_Character_o * c, MethodInfo * method)	48
/// This hook is used for sound events that are synced with animations.
/// We are specifically using it for critical charge events, which is a special animation and sound effect that plays when a character is about to do a critical attack.
#[skyline::hook(offset = 0x25f1c20)]
pub fn my_start(this: *const u8, character: &Character, method_info: OptionalMethod) {
    call_original!(this, character, method_info);
    do_vibrate(|| {
        handle_my_start(character);
    });
}

pub fn handle_my_start(character: &Character) {
    // unsafe {
        if let Some(event_name) = unsafe { animationevent_get_stringparameter(character.fields.playing_event, None) } {
            let event_name = event_name.to_string();
            vibe_log!("{}: my start event.", event_name);

            // First, check for a custom vibration.
            let cobalt_name = format!("Cobalt_{}", event_name);

            if let Some(ref event) = get_vibration_event(cobalt_name.as_str()) {
                vibe_log!("{}: Running custom vibration event at my_start.", cobalt_name);
                run_vibration_event(&event);
            } else {
                // Check for generic charge event (that we don't know about) based on the name of the event.
                let is_critical_charge = event_name.starts_with("SE_Critical") && event_name.ends_with("Charge");

                vibe_log!("{}: is_critical_charge: {}", event_name, is_critical_charge);

                if is_critical_charge {
                    // We only want to feel vibrations if we are the ones doing the charging.
                    match get_vibration_judgement(character) {
                        VibrationJudgement::NoVibration => (),
                        VibrationJudgement::ShouldVibrate(_must_be_critical) => {
                            run_vibration_event_by_name("Cobalt_Critical_Charge");
                        },
                    }
                }
            }
        }
    // }
}
