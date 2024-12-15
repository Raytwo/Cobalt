use engage::{
    menu::{
        config::{ConfigBasicMenuItem, ConfigBasicMenuItemCommandMethods},
        BasicMenuResult, ConfigMenu,
    },
    mess::Mess,
    pad::Pad,
    util::get_instance,
};
use unity::prelude::*;

use super::{
    super::sequences::mainmenu::cobaltmenu::sequences::settings::{lod::LodSetting, render_scale::RenderScaleSetting},
    combatpopup::CombatPopupSettings,
    combatui::CombatUISettings,
    combatvibration::CombatVibrationsSettings,
    gamemode::GameModeSettings,
    open_anime_all_ondispose,
    ringpolishrumble::{get_ring_polish_item_key, RingPolishVibrationSetting},
    supportoutfit::SupportOutfitSetting,
};

pub struct CobaltSubmenu;

impl ConfigBasicMenuItemCommandMethods for CobaltSubmenu {
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let pad_instance = get_instance::<Pad>();

        // Check if A is pressed before executing any of this
        if pad_instance.npad_state.buttons.a() {
            if !pad_instance.old_buttons.a() {
                // Close the original Settings menu temporarily so it doesn't get drawn in the background
                this.menu.close_anime_all();

                // Initialize the menu
                ConfigMenu::create_bind(this.menu);

                let config_menu = this.menu.proc.child.as_mut().unwrap().cast_mut::<ConfigMenu<ConfigBasicMenuItem>>();

                // Register a OnDispose callback to restore the previous menu
                config_menu
                    .get_class_mut()
                    .get_virtual_method_mut("OnDispose")
                    .map(|method| method.method_ptr = open_anime_all_ondispose as _)
                    .unwrap();

                // Clear the buttons in the List so we can add our own
                config_menu.full_menu_item_list.clear();

                config_menu.add_item(ConfigBasicMenuItem::new_switch::<CombatVibrationsSettings>(localize::mess::get(
                    "combat_rumble_menu_item_name",
                )));
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<CombatPopupSettings>(localize::mess::get(
                    "combat_popup_name",
                )));
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<CombatUISettings>(localize::mess::get("combat_ui_name")));
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<RingPolishVibrationSetting>(localize::mess::get(
                    get_ring_polish_item_key(),
                )));
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<GameModeSettings>(
                    Mess::get("MID_GAMESTART_MODE_TITLE").to_string(),
                ));
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<SupportOutfitSetting>(localize::mess::get(
                    "support_outfit_item_name",
                )));
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<RenderScaleSetting>(localize::mess::get(
                    "render_scale_name",
                )));
                config_menu.add_item(ConfigBasicMenuItem::new_switch::<LodSetting>(localize::mess::get("lod_name")));

                BasicMenuResult::se_cursor()
            } else {
                BasicMenuResult::new()
            }
        } else {
            BasicMenuResult::new()
        }
    }

    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) {
        this.command_text = localize::mess::get("submenu_item_commandtext").into();
    }

    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) {
        this.help_text = localize::mess::get("cobalt_settings_submenu_item_helptext").into();
    }
}
