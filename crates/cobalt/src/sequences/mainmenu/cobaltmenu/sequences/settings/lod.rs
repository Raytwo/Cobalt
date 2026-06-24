use std::sync::{LazyLock, RwLock};

use engage::root::configbasicmenuitem::{ConfigBasicMenuItem, IConfigBasicMenuItem, IConfigBasicMenuItemMethods};
use unity::OptionalMethod;

use engage::{BasicMenuResult, ConfigBasicMenuItemSwitchMethods};

use crate::{
    graphics::lod::lod_hook,
    sequences::mainmenu::cobaltmenu::util::{read_from_path, write_to_path},
};

pub const LOD_PATH: &str = "sd:/engage/config/lod";

pub static CURRENT_LOD: LazyLock<RwLock<f32>> = LazyLock::new(|| {
    RwLock::new(read_from_path(LOD_PATH).unwrap_or(1.0))
});

pub struct LodSetting;

impl ConfigBasicMenuItemSwitchMethods for LodSetting {
    extern "C" fn custom_call(this: ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let mut current_lod = CURRENT_LOD.write().unwrap();

        let result = ConfigBasicMenuItem::change_key_value_2(*current_lod, 0.1, 3.0, 0.1);

        if *current_lod != result {
            *current_lod = result;
            drop(current_lod);

            write_to_path(LOD_PATH, &format!("{:.1}", result));
            lod_hook(result, result, None);

            Self::set_command_text(this, None);
            this.update_text();

            BasicMenuResult::se_cursor()
        } else {
            BasicMenuResult::new()
        }
    }

    extern "C" fn set_command_text(this: ConfigBasicMenuItem, _method_info: OptionalMethod) {
        this.set_m_command_text(format!("{:.1}", CURRENT_LOD.read().unwrap()).into());
    }

    extern "C" fn set_help_text(this: ConfigBasicMenuItem, _method_info: OptionalMethod) {
        this.set_m_help_text(localize::mess::get("lod_helptext").into());
    }
}
