use std::sync::{LazyLock, RwLock};

use unity::prelude::*;

use engage::menu::{
    config::{ConfigBasicMenuItem, ConfigBasicMenuItemSwitchMethods},
    BasicMenuResult,
};

pub static TOGGLE: LazyLock<RwLock<bool>> = LazyLock::new(|| RwLock::new(std::path::Path::new("sd:/engage/config/render_scale_enabled").exists()));

pub struct ToggleRenderScaleSetting;

impl ConfigBasicMenuItemSwitchMethods for ToggleRenderScaleSetting {

    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let toggle = *TOGGLE.read().unwrap();
        let result = ConfigBasicMenuItem::change_key_value_b(toggle);

        if toggle != result {
            if result {
                std::fs::File::create("sd:/engage/config/render_scale_enabled").expect("Could not create the Render Scale Toggle configuration file");
            } else {
                std::fs::remove_file("sd:/engage/config/render_scale_enabled").expect("Could not delete the Render Scale Toggle configuration file");
            }

            *TOGGLE.write().unwrap() = result;
            
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();

            BasicMenuResult::se_cursor()
        } else {
            BasicMenuResult::new()
        }
    }

    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) {
        if *TOGGLE.read().unwrap() {
            this.command_text = localize::mess::get("command_text_on").into();
        } else {
            this.command_text = localize::mess::get("command_text_off").into();
        }
    }

    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) {
        if *TOGGLE.read().unwrap() {
            this.help_text = localize::mess::get("render_scale_toggle_enabled_helptext").into();
        } else {
            this.help_text = localize::mess::get("render_scale_toggle_disabled_helptext").into();
        }
    }
}
