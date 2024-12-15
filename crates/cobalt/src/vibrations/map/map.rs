use crate::vibrations::queue_handlers::run_vibration_event_by_name;
use crate::vibrations::util::do_vibrate;
use unity::prelude::*;

use unity::engine::Vector3;

#[skyline::hook(offset = 0x2291990)]
pub fn app_game_sound_hit(
    position: Vector3<f32>,
    attack_type: i32,
    damage_level: DamageLevel,
    unit_material: *const u8,
    method_info: OptionalMethod,
) {
    do_vibrate(|| handle_app_game_sound_hit(damage_level));
    call_original!(position, attack_type, damage_level, unit_material, method_info);
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum DamageLevel {
    None,
    Low,
    Middle,
    High,
    Efficacy,
    Critical,
}

/// Handle the vibrations that happen on the map level
/// TODO: Differentiate between ally and enemy heals
pub fn handle_app_game_sound_hit(damage_level: DamageLevel) {
    use DamageLevel::*;
    let damage_vibration_event_name = match damage_level {
        None => "Cobalt_Map_DamageLevel_None",
        Low => "Cobalt_Map_DamageLevel_Low",
        Middle => "Cobalt_Map_DamageLevel_Middle",
        High => "Cobalt_Map_DamageLevel_High",
        Efficacy => "Cobalt_Map_DamageLevel_Efficacy",
        Critical => "Cobalt_Map_DamageLevel_Critical",
    };
    run_vibration_event_by_name(damage_vibration_event_name);
}
