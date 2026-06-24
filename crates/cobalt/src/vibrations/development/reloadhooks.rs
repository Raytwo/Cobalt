use engage::app::procinst::ProcInst;
use unity::OptionalMethod;

use crate::vibrationevents::load_vibration_event_data;

// Mod-dev only, reload XML here so mod authors can tweak vibration values without restarting
#[unity::hook("App", "SortieTopMenu", "CreateBind")]
pub fn sortie_top_menu_create_bind(proc: ProcInst, method_info: OptionalMethod) {
    call_original!(proc, method_info);
    load_vibration_event_data();
}

#[unity::hook("App", "MapSystemMenu", "CreateBind")]
pub fn map_system_menu_create_bind(proc: ProcInst, method_info: OptionalMethod) {
    call_original!(proc, method_info);
    load_vibration_event_data();
}
