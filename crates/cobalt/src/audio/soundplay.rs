use engage::app::mapmind::{IMapMindMethods, MapMind};
use engage::app::singletonclass_1::ISingletonClass_1Methods;
use engage::app::unit::IUnitMethods;
use engage::combat::character::Character;
use unity::prelude::*;

use crate::audio::{
    akmarkercallbackinfo_get_strlabel, combat_character_get_sound, combat_charactersound_play_voice,
    gamesound::gamesound_unitpickvoice, COBALT_EVENT_MARKER_PREFIX, ORIGINAL_SUFFIX, PLAY_ORIGINAL_V_PICK,
    PREVIOUS_LAST_PICK_VOICE, UNSAFE_CHARACTER_PTR,
};

use super::wwise::AkCallbackType;

// App.SoundWwise.SoundPlay$$PostEventCallback	71021f3a50	void App.SoundWwise.SoundPlay$$PostEventCallback(App_SoundWwise_SoundPlay_o * __this, Il2CppObject * cookie, int32_t type, AkCallbackInfo_o * callbackInfo, MethodInfo * method)	1760
#[skyline::hook(offset = 0x21f3a50)]
pub fn soundplay_posteventcallback(
    this: IlInstance,
    cookie: IlInstance,
    callback_type: AkCallbackType,
    callback_info: IlInstance,
    method_info: OptionalMethod,
) {
    if callback_type == AkCallbackType::Marker {
        let marker_label = unsafe { akmarkercallbackinfo_get_strlabel(callback_info, None) };
        // println!("[SoundPlay] Marker label: {}", marker_label.to_string());

        let label = marker_label.to_string();
        if label.starts_with(COBALT_EVENT_MARKER_PREFIX) {
            // println!("Entered CobaltEvent block");
            let event_name = label.trim_start_matches(COBALT_EVENT_MARKER_PREFIX);

            if event_name == "V_Pick_Original" {
                // println!("V_Pick_Original event detected");
                let my_unit = MapMind::get_instance().get_unit();
                unsafe {
                    my_unit.set_last_pick_voice(PREVIOUS_LAST_PICK_VOICE);
                    PLAY_ORIGINAL_V_PICK.store(true, std::sync::atomic::Ordering::Relaxed);
                    gamesound_unitpickvoice(my_unit, None);
                }
                // other non-V_Pick events
                // we only support so-called "Original" events for now
            } else if event_name.ends_with(ORIGINAL_SUFFIX) {
                unsafe {
                    if !UNSAFE_CHARACTER_PTR.is_null() {
                        let character = Character::from_il_instance(IlInstance::from_raw(UNSAFE_CHARACTER_PTR));
                        let sound = combat_character_get_sound(character, None);
                        combat_charactersound_play_voice(sound, event_name.into(), None);
                        UNSAFE_CHARACTER_PTR = std::ptr::null_mut();
                    }
                }
            }
        }
    }
    call_original!(this, cookie, callback_type, callback_info, method_info)
}
