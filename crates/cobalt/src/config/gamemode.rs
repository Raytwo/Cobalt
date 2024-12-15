use unity::prelude::*;

use engage::{
    gameuserdata::{GameMode, GameUserData},
    menu::{
        config::{ConfigBasicMenuItem, ConfigBasicMenuItemSwitchMethods},
        BasicMenuResult,
    },
    mess::Mess,
};

pub struct GameModeSettings;

impl ConfigBasicMenuItemSwitchMethods for GameModeSettings {
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        // Get its current boolean value
        let mode = GameUserData::get_game_mode();

        let result: GameMode = ConfigBasicMenuItem::change_key_value_i(mode as i32, 0, 2, 1).into();

        if mode != result {
            GameUserData::set_game_mode(result);

            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();

            BasicMenuResult::se_cursor()
        } else {
            BasicMenuResult::new()
        }
    }

    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) {
        let label = match GameUserData::get_game_mode() {
            GameMode::Casual => Mess::get("MID_SYS_Mode_Casual"),
            GameMode::Classic => Mess::get("MID_SYS_Mode_Classic"),
            GameMode::Phoenix => Mess::get("MID_SYS_Mode_Phoenix"),
        };

        this.command_text = label;
    }

    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) {
        let label = match GameUserData::get_game_mode() {
            GameMode::Casual => Mess::get("MID_H_Mode_Casual"),
            GameMode::Classic => {
                let label = Mess::get("MID_H_Mode_Classic");
                label.replace("\n", " ")
            },
            GameMode::Phoenix => Mess::get("MID_H_Mode_Phoenix"),
        };

        this.help_text = label;
    }
}
