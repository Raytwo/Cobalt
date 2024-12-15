use engage::menu::{
    config::{ConfigBasicMenuItem, ConfigBasicMenuItemSwitchMethods},
    BasicMenuResult,
};
use unity::prelude::*;

use crate::{
    graphics::lod::lod_hook,
    sequences::mainmenu::cobaltmenu::util::{read_from_path, write_to_path},
};

pub const LOD_PATH: &str = "sd:/engage/config/lod";
pub static mut CURRENT_LOD: f32 = 1.0;

pub struct LodSetting;

impl ConfigBasicMenuItemSwitchMethods for LodSetting {
    fn init_content(_this: &mut ConfigBasicMenuItem) {
        unsafe {
            CURRENT_LOD = get_lod_with_default();
        };
    }

    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let current_lod = unsafe { CURRENT_LOD };

        let result = ConfigBasicMenuItem::change_key_value_f(current_lod, 0.1, 3.0, 0.1);

        if current_lod != result {
            save_lod(result);
            Self::set_command_text(this, None);
            this.update_text();
            BasicMenuResult::se_cursor()
        } else {
            BasicMenuResult::new()
        }
    }

    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) {
        unsafe {
            this.command_text = format!("{:.1}", CURRENT_LOD).into();
        }
    }

    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) {
        this.help_text = localize::mess::get("lod_helptext").into()
    }
}

pub fn get_lod() -> Option<f32> {
    read_from_path(LOD_PATH)
}

pub fn get_lod_with_default() -> f32 {
    // TODO: Figure out what the default actually is!
    get_lod().unwrap_or(1.0)
}

pub fn save_lod(lod: f32) {
    unsafe {
        CURRENT_LOD = lod;
    }
    write_to_path(LOD_PATH, &format!("{:.1}", lod));
    lod_hook(lod, lod, None);
}
