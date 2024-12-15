use unity::prelude::*;

use engage::{
    gamevariable::GameVariableManager,
    menu::{
        config::{ConfigBasicMenuItem, ConfigBasicMenuItemSwitchMethods},
        BasicMenuResult,
    },
};

pub const SUPPORT_OUTFIT_KEY: &str = "G_SUPPORT_OUTFIT";
pub struct SupportOutfitSetting;

impl ConfigBasicMenuItemSwitchMethods for SupportOutfitSetting {
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        // Attempt to make the variable entry if it doesn't exist
        GameVariableManager::make_entry_norewind(SUPPORT_OUTFIT_KEY, 0);
        // Get its current boolean value
        let active = GameVariableManager::get_bool(SUPPORT_OUTFIT_KEY);

        let result = ConfigBasicMenuItem::change_key_value_b(active);

        if active != result {
            if result {
                GameVariableManager::set_bool(SUPPORT_OUTFIT_KEY, true);
            } else {
                GameVariableManager::set_bool(SUPPORT_OUTFIT_KEY, false);
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
        if GameVariableManager::get_bool(SUPPORT_OUTFIT_KEY) {
            this.command_text = localize::mess::get("support_outfit_item_enabled_commandtext").into();
        } else {
            this.command_text = localize::mess::get("support_outfit_item_disabled_commandtext").into();
        }
    }

    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) {
        if GameVariableManager::get_bool(SUPPORT_OUTFIT_KEY) {
            this.help_text = localize::mess::get("support_outfit_item_enabled_helptext").into();
        } else {
            this.help_text = localize::mess::get("support_outfit_item_disabled_helptext").into();
        }
    }
}