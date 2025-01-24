use engage::{gameuserdata::GameMode, menu::savedata::{SaveDataMenuItemContent, SaveDataMenuMenuItem}, mess::Mess, proc::ProcInstFields};
use unity::prelude::*;

use crate::api::events::{publish_system_event, SystemEvent};

// App.SaveDataMenu.MenuItemContent$$SetupByMenuItem(App_SaveDataMenu_MenuItemContent_o *__this,App_SaveDataMenu_MenuItem_o *menuItem,MethodInfo *method)
#[skyline::hook(offset = 0x1d66470)]
pub fn savedatamenu_setupbymenuitem(this: &mut SaveDataMenuItemContent, menu_item: &mut SaveDataMenuMenuItem, method_info: OptionalMethod) {
    call_original!(this, menu_item, method_info);

    // Phoenix
    if let Some(header) = &menu_item.save_data_header_handle.header {
        if header.gamemode == GameMode::Phoenix {
            this.mode_text.set_text(Mess::get("MID_SYS_Mode_Phoenix"), true);
            this.game_mode_image.set_color(0.69, 0.043, 0.412, 1.0);
        }
    }
}

#[unity::class("App", "GameSaveData")]
pub struct GameSaveData {
    ty: i32,
    index: i32,
    from_type: i32,
    from_index: i32,
    header: &'static (),
    is_success: bool,
    is_exclude_header_and_time: bool,
}

#[unity::class("App.GameSaveData", "ProcBase")]
pub struct GameSaveDataProcBase {
    proc: ProcInstFields,
    game_message: &'static (),
    save_data: &'static GameSaveData,
    // ...
}

#[skyline::hook(offset = 0x1e69ef0)]
pub fn gamesavedata_procread_deserialize(this: &GameSaveDataProcBase, method_info: OptionalMethod) {
    call_original!(this, method_info);
    publish_system_event(SystemEvent::SaveLoaded { ty: this.save_data.ty, slot_id: this.save_data.index });
}