use std::sync::{LazyLock, RwLock};

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

pub static CURRENT_RENDER_SCALE: LazyLock<RwLock<f32>> = LazyLock::new(|| {
    RwLock::new(read_from_path(RENDER_SCALE_PATH).unwrap_or(0.9))
});

pub struct RenderScaleSetting;

impl ConfigBasicMenuItemSwitchMethods for RenderScaleSetting {
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let mut current_scale = CURRENT_RENDER_SCALE.write().unwrap();

        let result = ConfigBasicMenuItem::change_key_value_f(*current_scale, 0.1, 4.0, 0.1);

        if *current_scale != result {
            // Store the new scale in the RwLock then drop the guard so set_render_scale does not deadlock when trying to read the RwLock
            *current_scale = result;
            drop(current_scale);

            write_to_path(RENDER_SCALE_PATH, &format!("{:.1}", result));
            set_render_scale(result, None);
            Self::set_command_text(this, None);
            this.update_text();

            BasicMenuResult::se_cursor()
        } else {
            BasicMenuResult::new()
        }
    }

    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) {
        this.command_text = format!("{:.1}", CURRENT_RENDER_SCALE.read().unwrap()).into();
    }

    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) {
        this.help_text = localize::mess::get("render_scale_helptext").into()
    }
}

pub fn get_render_scale_with_default() -> f32 {
    // TODO: Get the actual render scale from the game by reading from GameParam, instead of just assuming 0.9.
    // See App.RenderManager$$PushRenderScale
    // But it seems there are two possible values, 0.9 and 0.85? Also multiple types of render scales.
    // get_render_scale().unwrap_or(0.9)
    0.9
}
