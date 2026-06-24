use std::sync::{LazyLock, RwLock};

use engage::root::configbasicmenuitem::{ConfigBasicMenuItem, IConfigBasicMenuItem, IConfigBasicMenuItemMethods};
use unity::OptionalMethod;

use engage::{BasicMenuResult, ConfigBasicMenuItemExt, ConfigBasicMenuItemSwitchMethods};

pub static TOGGLE: LazyLock<RwLock<bool>> =
    LazyLock::new(|| RwLock::new(std::path::Path::new("sd:/engage/config/render_scale_enabled").exists()));

pub struct ToggleRenderScaleSetting;

impl ConfigBasicMenuItemSwitchMethods for ToggleRenderScaleSetting {
    extern "C" fn custom_call(this: ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let toggle = *TOGGLE.read().unwrap();
        let result = ConfigBasicMenuItem::change_key_value_b(toggle);

        if toggle != result {
            if result {
                std::fs::File::create("sd:/engage/config/render_scale_enabled")
                    .expect("Could not create the Render Scale Toggle configuration file");
            } else {
                std::fs::remove_file("sd:/engage/config/render_scale_enabled")
                    .expect("Could not delete the Render Scale Toggle configuration file");
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

    extern "C" fn set_command_text(this: ConfigBasicMenuItem, _method_info: OptionalMethod) {
        let text = if *TOGGLE.read().unwrap() {
            localize::mess::get("command_text_on")
        } else {
            localize::mess::get("command_text_off")
        };
        this.set_m_command_text(text.into());
    }

    extern "C" fn set_help_text(this: ConfigBasicMenuItem, _method_info: OptionalMethod) {
        let text = if *TOGGLE.read().unwrap() {
            localize::mess::get("render_scale_toggle_enabled_helptext")
        } else {
            localize::mess::get("render_scale_toggle_disabled_helptext")
        };
        this.set_m_help_text(text.into());
    }
}
