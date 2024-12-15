use engage::menu::{
    config::{ConfigBasicMenuItem, ConfigBasicMenuItemSwitchMethods},
    BasicMenuResult,
};
use unity::prelude::*;

use crate::{
    graphics::render_scale::set_render_scale,
    sequences::mainmenu::cobaltmenu::util::{read_from_path, write_to_path},
};

pub const RENDER_SCALE_PATH: &str = "sd:/engage/config/render_scale";

pub struct RenderScaleSetting;
pub static mut CURRENT_RENDER_SCALE: f32 = 0.9;

impl ConfigBasicMenuItemSwitchMethods for RenderScaleSetting {
    fn init_content(_this: &mut ConfigBasicMenuItem) {
        unsafe {
            CURRENT_RENDER_SCALE = get_render_scale_with_default();
        };
    }

    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let current_scale = unsafe { CURRENT_RENDER_SCALE };

        let result = ConfigBasicMenuItem::change_key_value_f(current_scale, 0.1, 4.0, 0.1);

        if current_scale != result {
            save_render_scale(result);
            Self::set_command_text(this, None);
            this.update_text();
            BasicMenuResult::se_cursor()
        } else {
            BasicMenuResult::new()
        }
    }

    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) {
        unsafe {
            this.command_text = format!("{:.1}", CURRENT_RENDER_SCALE).into();
        }
    }

    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) {
        this.help_text = localize::mess::get("render_scale_helptext").into()
    }
}

pub fn get_render_scale() -> Option<f32> {
    read_from_path(RENDER_SCALE_PATH)
}

pub fn get_render_scale_with_default() -> f32 {
    // TODO: Get the actual render scale from the game by reading from GameParam, instead of just assuming 0.9.
    // See App.RenderManager$$PushRenderScale
    // But it seems there are two possible values, 0.9 and 0.85? Also multiple types of render scales.
    get_render_scale().unwrap_or(0.9)
}

pub fn save_render_scale(render_scale: f32) {
    unsafe {
        CURRENT_RENDER_SCALE = render_scale;
    }
    write_to_path(RENDER_SCALE_PATH, &format!("{:.1}", render_scale));
    set_render_scale(render_scale, None);
}
