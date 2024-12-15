use unity::prelude::*;

use engage::{
    gamevariable::GameVariableManager,
    godpool,
    menu::{
        config::{ConfigBasicMenuItem, ConfigBasicMenuItemSwitchMethods},
        BasicMenuResult,
    },
    vibrationmanager::{vibrate, FREQ_HIGH, FREQ_LOW},
};

pub const ENABLE_POLISH_VIBRATION_KEY: &str = "G_RING_POLISH_VIBRATION";
pub struct RingPolishVibrationSetting;

impl ConfigBasicMenuItemSwitchMethods for RingPolishVibrationSetting {
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        // Attempt to make the variable entry if it doesn't exist
        GameVariableManager::make_entry_norewind(ENABLE_POLISH_VIBRATION_KEY, 0);

        // Get its current boolean value
        let active = GameVariableManager::get_bool(ENABLE_POLISH_VIBRATION_KEY);

        let result = ConfigBasicMenuItem::change_key_value_b(active);

        if active != result {
            if result {
                GameVariableManager::set_bool(ENABLE_POLISH_VIBRATION_KEY, true);
                vibrate(0.15, 0.15, 0.10, 0.0, FREQ_LOW, FREQ_HIGH)
            } else {
                GameVariableManager::set_bool(ENABLE_POLISH_VIBRATION_KEY, false);
            }
            
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);

            this.update_text();

            BasicMenuResult::se_cursor()
        } else {
            BasicMenuResult::new()
        }
    }

    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) {
        if GameVariableManager::get_bool(ENABLE_POLISH_VIBRATION_KEY) {
            this.command_text = localize::mess::get("command_text_on").into();
        } else {
            this.command_text = localize::mess::get("command_text_off").into();
        }
    }

    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) {
        if GameVariableManager::get_bool(ENABLE_POLISH_VIBRATION_KEY) {
            this.help_text = localize::mess::get("rumble_menu_item_enabled_helptext").into();
        } else {
            this.help_text = localize::mess::get("rumble_menu_item_disabled_helptext").into();
        }
    }
}

pub fn get_ring_polish_item_key() -> &'static str {
    unsafe {
        if godpool::has_armlet(None) {
            "ring_dlc_polish_rumble_menu_item_name"
        } else {
            "ring_polish_rumble_menu_item_name"
        }
    }
}
