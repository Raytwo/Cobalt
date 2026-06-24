use engage::root::configbasicmenuitem::{ConfigBasicMenuItem, IConfigBasicMenuItem, IConfigBasicMenuItemMethods};
use unity::OptionalMethod;

use engage::{
    BasicMenuResult, ConfigBasicMenuItemExt, ConfigBasicMenuItemSwitchMethods, GameVariableManager,
};

pub const SUPPORT_OUTFIT_KEY: &str = "G_SUPPORT_OUTFIT";
pub struct SupportOutfitSetting;

impl ConfigBasicMenuItemSwitchMethods for SupportOutfitSetting {
    extern "C" fn custom_call(this: ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        GameVariableManager::make_entry_norewind(SUPPORT_OUTFIT_KEY, 0);
        let active = GameVariableManager::get_bool(SUPPORT_OUTFIT_KEY);

        let result = ConfigBasicMenuItem::change_key_value_b(active);

        if active != result {
            GameVariableManager::set_bool(SUPPORT_OUTFIT_KEY, result);
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();

            BasicMenuResult::se_cursor()
        } else {
            BasicMenuResult::new()
        }
    }

    extern "C" fn set_command_text(this: ConfigBasicMenuItem, _method_info: OptionalMethod) {
        let text = if GameVariableManager::get_bool(SUPPORT_OUTFIT_KEY) {
            localize::mess::get("support_outfit_item_enabled_commandtext")
        } else {
            localize::mess::get("support_outfit_item_disabled_commandtext")
        };
        this.set_m_command_text(text.into());
    }

    extern "C" fn set_help_text(this: ConfigBasicMenuItem, _method_info: OptionalMethod) {
        let text = if GameVariableManager::get_bool(SUPPORT_OUTFIT_KEY) {
            localize::mess::get("support_outfit_item_enabled_helptext")
        } else {
            localize::mess::get("support_outfit_item_disabled_helptext")
        };
        this.set_m_help_text(text.into());
    }
}
