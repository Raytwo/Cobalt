use engage::app::{
    gamemode::GameMode,
    gameuserdata::{GameUserData, IGameUserDataMethods},
    singletonclass_1::ISingletonClass_1Methods,
};
use engage::root::configbasicmenuitem::{ConfigBasicMenuItem, IConfigBasicMenuItem, IConfigBasicMenuItemMethods};
use unity::OptionalMethod;

use engage::{BasicMenuResult, ConfigBasicMenuItemSwitchMethods, Mess};

pub struct GameModeSettings;

impl ConfigBasicMenuItemSwitchMethods for GameModeSettings {
    extern "C" fn custom_call(this: ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let mode = GameUserData::get_instance().get_game_mode();
        let raw = ConfigBasicMenuItem::change_key_value(mode.value, 0, 2, 1);
        if !(0..=2).contains(&raw) {
            return BasicMenuResult::new();
        }
        let result = GameMode { value: raw };

        if mode != result {
            GameUserData::get_instance().set_game_mode(result);
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();

            BasicMenuResult::se_cursor()
        } else {
            BasicMenuResult::new()
        }
    }

    extern "C" fn set_command_text(this: ConfigBasicMenuItem, _method_info: OptionalMethod) {
        let label = match GameUserData::get_instance().get_game_mode().value {
            0 => Mess::get("MID_SYS_Mode_Casual"),
            1 => Mess::get("MID_SYS_Mode_Classic"),
            2 => Mess::get("MID_SYS_Mode_Phoenix"),
            _ => Mess::get("MID_SYS_Mode_Casual"),
        };
        this.set_m_command_text(label);
    }

    extern "C" fn set_help_text(this: ConfigBasicMenuItem, _method_info: OptionalMethod) {
        let label = match GameUserData::get_instance().get_game_mode().value {
            0 => Mess::get("MID_H_Mode_Casual").to_string(),
            1 => Mess::get("MID_H_Mode_Classic").to_string().replace('\n', " "),
            2 => Mess::get("MID_H_Mode_Phoenix").to_string(),
            _ => Mess::get("MID_H_Mode_Casual").to_string(),
        };
        this.set_m_help_text(label.into());
    }
}
