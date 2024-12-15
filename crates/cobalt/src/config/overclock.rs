use unity::prelude::*;

use engage::menu::{
    config::{ConfigBasicMenuItem, ConfigBasicMenuItemSwitchMethods},
    BasicMenuResult,
};

pub static mut OVERCLOCK: bool = false;


extern "C" {
    
    #[link_name = "_ZN2nn2oe27SetPerformanceConfigurationENS0_15PerformanceModeEi"]
    pub fn set_performance_configuration_2(mode: i32, configuration: i32);
}

pub struct OverclockSetting;

impl ConfigBasicMenuItemSwitchMethods for OverclockSetting {
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let skip = unsafe { OVERCLOCK };
        let result = ConfigBasicMenuItem::change_key_value_b(skip);

        if skip != result {
            if result {
                unsafe { set_performance_configuration_2(1, 0x00020004); }
            } else {
                unsafe { set_performance_configuration_2(1, 0x00020003); }
            }

            unsafe { OVERCLOCK = result }
            
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();

            BasicMenuResult::se_cursor()
        } else {
            BasicMenuResult::new()
        }
    }

    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) {
        if unsafe { OVERCLOCK } {
            this.command_text = localize::mess::get("command_text_on").into();
        } else {
            this.command_text = localize::mess::get("command_text_off").into();
        }
    }

    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) {
        if unsafe { OVERCLOCK } {
            this.help_text = "GPU performance set to 384 MHz".into();
        } else {
            this.help_text = "GPU performance set to 307 MHz".into();
        }
    }
}
