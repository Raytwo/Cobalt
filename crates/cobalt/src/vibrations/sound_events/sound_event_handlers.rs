use engage::combat::character::Character;
use engage::app::mapmind::{IMapMindMethods, MapMind};
use engage::app::singletonclass_1::ISingletonClass_1Methods;
use engage::app::unit::IUnitMethods;
use unity::prelude::*;
use unity::Cast;

use crate::vibrations::queue_handlers::run_vibration_event_by_name;
use crate::vibrations::util::do_vibrate;

#[unity::hook("App", "SoundManager", "PostEvent")]
pub fn app_sound_manager_post_event(
    this: *const u8,
    event_name: Il2CppString,
    character: *const u8,
    is_get_position: bool,
    method_info: OptionalMethod,
) -> *const u8 {
    do_vibrate(|| process_sound_event_name(event_name));
    call_original!(this, event_name, character, is_get_position, method_info)
}

#[unity::hook("App", "SoundManager", "PostEvent")]
pub fn app_sound_manager_post_event_2(
    this: *const u8,
    event_name: Il2CppString,
    game_object: *const u8,
    character: Character,
    is_get_position: bool,
    method_info: OptionalMethod,
) -> *const u8 {
    do_vibrate(|| process_sound_event_name(event_name));
    call_original!(this, event_name, game_object, character, is_get_position, method_info)
}

/// Get the sound event name and start processing it for vibrations.
///
/// The most generic vibration event handler.
/// These are all the vibrations that I didn't want to bother finding hooks for.
/// As a side effect, they also always apply whether friend or foe...
pub fn process_sound_event_name(event_name: Il2CppString) {
    if event_name.is_null() {
        return;
    }
    let name = event_name.to_rust_string();
    let remapped = check_unit_selection(&name).unwrap_or(&name);
    run_vibration_event_by_name(remapped);
}

/// Special logic for what vibration to play when a unit is selected or released.
/// Depends on if the unit has an engage partner or not.
fn check_unit_selection(event_name: &str) -> Option<&str> {
    match event_name {
        "UnitTouch" => {
            let my_unit = MapMind::get_instance().get_unit();
            if my_unit.is_null() {
                return None;
            }
            match (my_unit.is_engage_owner(), my_unit.is_engaging_2()) {
                (false, _) => Some("UnitTouch_NotEngageOwner"),
                    // do nothing, we're covered by the god appearing
                (true, false) => None,
                (true, true) => Some("UnitTouch_NotEngageOwner"),
            }
        }
        "UnitRelease" => {
            let my_unit = MapMind::get_instance().get_unit();
            if my_unit.is_null() {
                return None;
            }
            match (my_unit.is_engage_owner(), my_unit.is_engaging_2()) {
                (false, _) => Some("UnitRelease_NotEngageOwner"),
                (true, false) => None,
                (true, true) => Some("UnitRelease_Engaged"),
            }
        }
        _ => None,
    }
}
