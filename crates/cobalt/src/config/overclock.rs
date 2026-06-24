use engage::root::configbasicmenuitem::{ConfigBasicMenuItem, IConfigBasicMenuItem, IConfigBasicMenuItemMethods};
use unity::OptionalMethod;

use engage::{BasicMenuResult, ConfigBasicMenuItemExt, ConfigBasicMenuItemSwitchMethods};

pub static mut OVERCLOCK: bool = false;

extern "C" {
    #[link_name = "_ZN2nn2oe27SetPerformanceConfigurationENS0_15PerformanceModeEi"]
    pub fn set_performance_configuration_2(mode: i32, configuration: i32);
}

pub struct OverclockSetting;

impl ConfigBasicMenuItemSwitchMethods for OverclockSetting {
    extern "C" fn custom_call(this: ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let current = unsafe { OVERCLOCK };
        let result = ConfigBasicMenuItem::change_key_value_b(current);

        if current != result {
            unsafe {
                let config = if result { 0x00020004 } else { 0x00020003 };
                set_performance_configuration_2(1, config);
                OVERCLOCK = result;
            }

            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();

            BasicMenuResult::se_cursor()
        } else {
            BasicMenuResult::new()
        }
    }

    extern "C" fn set_command_text(this: ConfigBasicMenuItem, _method_info: OptionalMethod) {
        let text = if unsafe { OVERCLOCK } {
            localize::mess::get("command_text_on")
        } else {
            localize::mess::get("command_text_off")
        };
        this.set_m_command_text(text.into());
    }

    extern "C" fn set_help_text(this: ConfigBasicMenuItem, _method_info: OptionalMethod) {
        let text = if unsafe { OVERCLOCK } {
            "GPU performance set to 384 MHz"
        } else {
            "GPU performance set to 307 MHz"
        };
        this.set_m_help_text(text.into());
    }
}
