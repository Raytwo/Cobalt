use crate::vibrationevents::load_vibration_event_data;
use engage::proc::ProcInst;
use unity::prelude::*;

// App.SortieTopMenu$$CreateBind	71024eb880	void App.SortieTopMenu$$CreateBind(App_ProcInst_o * super, MethodInfo * method)	1380
/// Loads vibration data when the sortie system menu is opened.
/// Only useful for vibration development purposes
#[skyline::hook(offset = 0x24eb880)]
pub fn sortie_top_menu_create_bind(proc: &ProcInst, method_info: OptionalMethod) {
    call_original!(proc, method_info);
    load_vibration_event_data();
}

// App.MapSystemMenu$$CreateBind	7101f52490	void App.MapSystemMenu$$CreateBind(App_ProcInst_o * super, MethodInfo * method)	812
/// Loads vibration data when the map system menu is opened.
/// Only useful for vibration development purposes
#[skyline::hook(offset = 0x1f52490)]
pub fn map_system_menu_create_bind(proc: &ProcInst, method_info: OptionalMethod) {
    call_original!(proc, method_info);
    load_vibration_event_data();
}
