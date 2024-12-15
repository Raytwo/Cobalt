use unity::prelude::*;

use engage::{
    gamevariable::GameVariableManager,
    menu::{
        config::{ConfigBasicMenuItem, ConfigBasicMenuItemSwitchMethods},
        BasicMenuResult,
    },
};

pub const DISABLE_COMBAT_POPUPS_KEY: &str = "G_DISABLE_COMBAT_POPUPS";
pub struct CombatPopupSettings;

impl ConfigBasicMenuItemSwitchMethods for CombatPopupSettings {
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        // Attempt to make the variable entry if it doesn't exist
        GameVariableManager::make_entry_norewind(DISABLE_COMBAT_POPUPS_KEY, 0);
        // Get its current boolean value
        let active = GameVariableManager::get_bool(DISABLE_COMBAT_POPUPS_KEY);

        let result = ConfigBasicMenuItem::change_key_value_b(active);

        if active != result {
            if result {
                GameVariableManager::set_bool(DISABLE_COMBAT_POPUPS_KEY, true);
            } else {
                GameVariableManager::set_bool(DISABLE_COMBAT_POPUPS_KEY, false);
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
        if GameVariableManager::get_bool(DISABLE_COMBAT_POPUPS_KEY) {
            this.command_text = localize::mess::get("command_text_off").into();
        } else {
            this.command_text = localize::mess::get("command_text_on").into();
        }
    }

    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) {
        if GameVariableManager::get_bool(DISABLE_COMBAT_POPUPS_KEY) {
            this.help_text = localize::mess::get("combat_popup_disabled_helptext").into();
        } else {
            this.help_text = localize::mess::get("combat_popup_enabled_helptext").into();
        }
    }
}
