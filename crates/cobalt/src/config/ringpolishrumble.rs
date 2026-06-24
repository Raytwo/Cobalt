use engage::app::godpool::GodPool;
use engage::root::configbasicmenuitem::{ConfigBasicMenuItem, IConfigBasicMenuItem, IConfigBasicMenuItemMethods};
use unity::OptionalMethod;

use engage::{
    vibrate, BasicMenuResult, ConfigBasicMenuItemExt, ConfigBasicMenuItemSwitchMethods,
    GameVariableManager, FREQ_HIGH, FREQ_LOW,
};

pub const ENABLE_POLISH_VIBRATION_KEY: &str = "G_RING_POLISH_VIBRATION";
pub struct RingPolishVibrationSetting;

impl ConfigBasicMenuItemSwitchMethods for RingPolishVibrationSetting {
    extern "C" fn custom_call(this: ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        GameVariableManager::make_entry_norewind(ENABLE_POLISH_VIBRATION_KEY, 0);
        let active = GameVariableManager::get_bool(ENABLE_POLISH_VIBRATION_KEY);

        let result = ConfigBasicMenuItem::change_key_value_b(active);

        if active != result {
            GameVariableManager::set_bool(ENABLE_POLISH_VIBRATION_KEY, result);
            if result {
                vibrate(0.15, 0.15, 0.10, 0.0, FREQ_LOW, FREQ_HIGH);
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
        let text = if GameVariableManager::get_bool(ENABLE_POLISH_VIBRATION_KEY) {
            localize::mess::get("command_text_on")
        } else {
            localize::mess::get("command_text_off")
        };
        this.set_m_command_text(text.into());
    }

    extern "C" fn set_help_text(this: ConfigBasicMenuItem, _method_info: OptionalMethod) {
        let text = if GameVariableManager::get_bool(ENABLE_POLISH_VIBRATION_KEY) {
            localize::mess::get("rumble_menu_item_enabled_helptext")
        } else {
            localize::mess::get("rumble_menu_item_disabled_helptext")
        };
        this.set_m_help_text(text.into());
    }
}

pub fn get_ring_polish_item_key() -> &'static str {
    if GodPool::has_armlet() {
        "ring_dlc_polish_rumble_menu_item_name"
    } else {
        "ring_polish_rumble_menu_item_name"
    }
}
