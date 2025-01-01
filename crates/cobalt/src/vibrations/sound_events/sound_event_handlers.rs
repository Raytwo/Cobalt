use engage::{combat::Character, mapmind::MapMind};
use unity::prelude::*;

use crate::vibrations::{queue_handlers::run_vibration_event_by_name, util::do_vibrate};

#[skyline::hook(offset = 0x24f70b0)]
pub fn app_sound_manager_post_event(
    this: *const u8,
    event_name: Option<&Il2CppString>,
    character: *const u8,
    is_get_position: bool,
    method_info: OptionalMethod,
) -> *const u8 {
    // println!(
    //     "[SoundManager::PostEvent] Event name: {}",
    //     event_name.unwrap_or("[Blank]".into()).to_string()
    // );
    do_vibrate(|| {
        process_sound_event_name(event_name);
    });
    call_original!(this, event_name, character, is_get_position, method_info)
}

#[skyline::hook(offset = 0x24f7300)]
pub fn app_sound_manager_post_event_2(
    this: *const u8,
    event_name: Option<&Il2CppString>,
    game_object: *const u8,
    character: &Character,
    is_get_position: bool,
    method_info: OptionalMethod,
) -> *const u8 {
    // println!(
    //     "[SoundManager::PostEvent2] Event name: {}",
    //     event_name.unwrap_or("[Blank]".into()).to_string()
    // );
    do_vibrate(|| {
        process_sound_event_name(event_name);
    });
    call_original!(this, event_name, game_object, character, is_get_position, method_info)
}

/// Get the sound event name and start processing it for vibrations.
///
/// The most generic vibration event handler.
/// These are all the vibrations that I didn't want to bother finding hooks for.
/// As a side effect, they also always apply whether friend or foe...
pub fn process_sound_event_name(event_name: Option<&Il2CppString>) {
    if let Some(event_name) = event_name {
        let event_name = event_name.to_string();
        let event_name = check_unit_selection(&event_name).unwrap_or(&event_name);
        run_vibration_event_by_name(event_name);
    }
}

/// Special logic for what vibration to play when a unit is selected or released.
/// Depends on if the unit has an engage partner or not.
fn check_unit_selection(event_name: &str) -> Option<&str> {
    match event_name {
        "UnitTouch" => {
            let my_unit = MapMind::get_unit();
            let is_engage_owner = my_unit.is_engage_owner();
            let is_engaging = my_unit.is_engaging();
            match (is_engage_owner, is_engaging) {
                (false, _) => Some("UnitTouch_NotEngageOwner"),
                (true, false) => {
                    // do nothing, we're covered by the god appearing
                    None
                },
                (true, true) => Some("UnitTouch_NotEngageOwner"),
            }
        },
        "UnitRelease" => {
            let my_unit = MapMind::get_unit();
            let is_engage_owner = my_unit.is_engage_owner();
            let is_engaging = my_unit.is_engaging();
            match (is_engage_owner, is_engaging) {
                (false, _) => Some("UnitRelease_NotEngageOwner"),
                (true, false) => {
                    // do nothing, we're covered by the god disappearing
                    None
                },
                (true, true) => Some("UnitRelease_Engaged"),
            }
        },
        _ => None,
    }
}
