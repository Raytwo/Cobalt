use engage::root::configbasicmenuitem::{
    ConfigBasicMenuItem, IConfigBasicMenuItem, IConfigBasicMenuItemMethods,
};
use unity::OptionalMethod;

use engage::{BasicMenuResult, ConfigBasicMenuItemExt, ConfigBasicMenuItemSwitchMethods};

pub static mut SKIP: bool = false;

pub struct SkipOpeningSetting;

impl ConfigBasicMenuItemSwitchMethods for SkipOpeningSetting {
    fn init_content(_this: ConfigBasicMenuItem) {
        unsafe { SKIP = std::path::Path::new("sd:/engage/config/gop_skip").exists() };
    }

    extern "C" fn custom_call(this: ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let skip = unsafe { SKIP };
        let result = ConfigBasicMenuItem::change_key_value_b(skip);

        if skip != result {
            if result {
                std::fs::File::create("sd:/engage/config/gop_skip")
                    .expect("Could not create the Grand Opening Skip configuration file");
            } else {
                std::fs::remove_file("sd:/engage/config/gop_skip")
                    .expect("Could not delete the Grand Opening Skip configuration file");
            }

            unsafe { SKIP = result };

            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();

            BasicMenuResult::se_cursor()
        } else {
            BasicMenuResult::new()
        }
    }

    extern "C" fn set_command_text(this: ConfigBasicMenuItem, _method_info: OptionalMethod) {
        let text = if unsafe { SKIP } {
            localize::mess::get("command_text_on")
        } else {
            localize::mess::get("command_text_off")
        };
        this.set_m_command_text(text.into());
    }

    extern "C" fn set_help_text(this: ConfigBasicMenuItem, _method_info: OptionalMethod) {
        let text = if unsafe { SKIP } {
            localize::mess::get("skip_opening_helptext")
        } else {
            localize::mess::get("play_opening_helptext")
        };
        this.set_m_help_text(text.into());
    }
}
