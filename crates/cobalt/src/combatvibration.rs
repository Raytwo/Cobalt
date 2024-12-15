use engage::{gamevariable::GameVariableManager, mapmind::MapMind, vibrationmanager::vibrate};
use unity::prelude::*;

use crate::config::combatvibration::COMBAT_VIBRATION_KEY;

// bool App.MapSequenceHuman$$PickUnitTick
// (App_MapSequenceHuman_o *__this,int32_t label,MethodInfo *method)
#[unity::hook("App", "MapSequenceHuman", "PickUnitTick")]
pub fn pick_unit_tick(this: *const u8, label: i32, method_info: OptionalMethod) {
    call_original!(this, label, method_info);
    let my_unit = MapMind::get_unit();
    if my_unit.is_engaging() {
        // TODO: Find the right value such that this doesn't feel annoying.
        // TODO: Ensure the engaging vibration survive opening of menus (this tick is only when selecting and moving the unit).
        magic_wand();
    }
}

/// Supposed to represent the vibrating power of an engaged unit.
/// But actually just feels annoying.
fn magic_wand() {
    let vibration_enabled = GameVariableManager::get_bool(COMBAT_VIBRATION_KEY);
    if !vibration_enabled {
        return;
    }
    vibrate(0.01666666666, 0.04, 0.01, 0.05, 20.0, 50.0);
}

// App.MapUnitCommandMenu$$Tick	710202bca0	void App.MapUnitCommandMenu$$Tick(App_MapUnitCommandMenu_o * __this, MethodInfo * method)	220
#[unity::hook("App", "MapUnitCommandMenu", "Tick")]
pub fn app_map_unit_command_menu_tick(this: *const u8, method_info: OptionalMethod) {
    call_original!(this, method_info);
    let my_unit = MapMind::get_unit();
    if my_unit.is_engaging() {
        // TODO: Find the right value such that this doesn't feel annoying.
        // TODO: Ensure the engaging vibration survive opening of menus (this tick is only when selecting and moving the unit).
        magic_wand();
    }
}
