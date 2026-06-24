use engage::root::configbasicmenuitem::{ConfigBasicMenuItem, IConfigBasicMenuItem, IConfigBasicMenuItemMethods};
use unity::OptionalMethod;

use engage::{
    BasicMenuResult, ConfigBasicMenuItemExt, ConfigBasicMenuItemSwitchMethods, GameVariableManager,
};

pub const DISABLE_COMBAT_UI_KEY: &str = "G_DISABLE_COMBAT_UI";
pub struct CombatUISettings;

impl ConfigBasicMenuItemSwitchMethods for CombatUISettings {
    extern "C" fn custom_call(this: ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        GameVariableManager::make_entry_norewind(DISABLE_COMBAT_UI_KEY, 0);
        let active = GameVariableManager::get_bool(DISABLE_COMBAT_UI_KEY);

        let result = ConfigBasicMenuItem::change_key_value_b(active);

        if active != result {
            GameVariableManager::set_bool(DISABLE_COMBAT_UI_KEY, result);
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();

            BasicMenuResult::se_cursor()
        } else {
            BasicMenuResult::new()
        }
    }

    extern "C" fn set_command_text(this: ConfigBasicMenuItem, _method_info: OptionalMethod) {
        let text = if GameVariableManager::get_bool(DISABLE_COMBAT_UI_KEY) {
            localize::mess::get("command_text_off")
        } else {
            localize::mess::get("command_text_on")
        };
        this.set_m_command_text(text.into());
    }

    extern "C" fn set_help_text(this: ConfigBasicMenuItem, _method_info: OptionalMethod) {
        let text = if GameVariableManager::get_bool(DISABLE_COMBAT_UI_KEY) {
            localize::mess::get("combat_ui_disabled_helptext")
        } else {
            localize::mess::get("combat_ui_enabled_helptext")
        };
        this.set_m_help_text(text.into());
    }
}
