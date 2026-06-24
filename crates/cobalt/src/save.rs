use engage::app::gamemode::GameMode;
use engage::app::gamesavedata::{
    GameSaveData_ProcBase, IGameSaveDataMethods, IGameSaveData_ProcBase,
};
use engage::app::gamesavedataheader::IGameSaveDataHeaderMethods;
use engage::app::gamesavedataheaderreader::IGameSaveDataHeaderReader_HandleMethods;
use engage::app::savedatamenu::{
    ISaveDataMenu_MenuItem, ISaveDataMenu_MenuItemContent, SaveDataMenu_MenuItem,
    SaveDataMenu_MenuItemContent,
};
use engage::tm_pro::tmp_text::ITMP_TextMethods;
use engage::unity_engine::color::Color;
use engage::unity_engine::ui::graphic::IGraphicMethods;
use unity::{Cast, OptionalMethod};

use crate::api::events::{publish_system_event, SystemEvent};
use engage::Mess;

#[unity::hook("App", "SaveDataMenu.MenuItemContent", "SetupByMenuItem")]
pub fn savedatamenu_setupbymenuitem(
    this: SaveDataMenu_MenuItemContent,
    menu_item: SaveDataMenu_MenuItem,
    method_info: OptionalMethod,
) {
    call_original!(this, menu_item, method_info);

    let header_handle = menu_item.m_save_data_header_handle();
    if header_handle.is_null() {
        return;
    }
    let header = header_handle.get_header();
    if header.is_null() {
        return;
    }

    if header.get_game_mode() == GameMode::phoenix() {
        this.m_mode_text().set_text(Mess::get("MID_SYS_Mode_Phoenix"));
        this.m_game_mode_image().set_color(Color { r: 0.69, g: 0.043, b: 0.412, a: 1.0 });
    }
}

#[unity::hook("App", "GameSaveData.ProcRead", "Deserialize")]
pub fn gamesavedata_procread_deserialize(this: GameSaveData_ProcBase, method_info: OptionalMethod) {
    call_original!(this, method_info);
    let save_data = this.m_save_data();
    publish_system_event(SystemEvent::SaveLoaded {
        ty: save_data.get_type().value,
        slot_id: save_data.get_index(),
    });
}
