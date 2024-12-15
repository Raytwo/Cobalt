use engage::{gameuserdata::GameMode, menu::savedata::{SaveDataMenuItemContent, SaveDataMenuMenuItem}, mess::Mess};
use unity::prelude::*;

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