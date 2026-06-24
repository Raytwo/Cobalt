use engage::app::pad::Pad;
use engage::nn::hid::npadbutton::NpadButton;
use engage::app::basicmenu::{BasicMenu, IBasicMenu};
use engage::app::procinst::ProcInst;
use engage::root::configbasicmenuitem::{ConfigBasicMenuItem, IConfigBasicMenuItem};
use engage::root::configmenu::ConfigMenu;
use engage::system::collections::generic::list_1::IList_1Methods as IListMethods;
use unity::{Cast, OptionalMethod};

use engage::{
    restore_parent_on_dispose, BasicMenuExt, BasicMenuResult, ConfigBasicMenuItemCommandMethods,
    ConfigBasicMenuItemExt, Mess,
};

use super::{
    super::sequences::mainmenu::cobaltmenu::sequences::settings::{
        lod::LodSetting,
        render_scale::RenderScaleSetting,
        render_scale_toggle::ToggleRenderScaleSetting,
    },
    combatpopup::CombatPopupSettings,
    combatui::CombatUISettings,
    combatvibration::CombatVibrationsSettings,
    gamemode::GameModeSettings,
    ringpolishrumble::{get_ring_polish_item_key, RingPolishVibrationSetting},
    supportoutfit::SupportOutfitSetting,
};

pub struct CobaltSubmenu;

impl ConfigBasicMenuItemCommandMethods for CobaltSubmenu {
    extern "C" fn custom_call(this: ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        use engage::app::basicmenuitem::IBasicMenuItemMethods;
        use engage::app::procinst::IProcInstMethods;
        if !Pad::is_trigger(NpadButton::a()) {
            return BasicMenuResult::new();
        }

        let parent_menu = this.get_menu();
        BasicMenuExt::close_anime_all(parent_menu);

        let parent_menu_proc: ProcInst = parent_menu.into();
        ConfigMenu::create_bind(parent_menu_proc);

        let child_proc = parent_menu_proc.get_child();
        if child_proc.is_null() {
            return BasicMenuResult::new();
        }
        let config_menu = unsafe { child_proc.cast::<ConfigMenu>() };

        config_menu.override_class().override_virtual_method(
            "OnDispose",
            unity::method_info!(restore_parent_on_dispose, 0),
        );

        let as_menu = unsafe { config_menu.cast::<BasicMenu>() };
        as_menu.m_full_menu_item_list().clear();

        as_menu.add_item(ConfigBasicMenuItem::new_switch::<CombatVibrationsSettings>(
            localize::mess::get("combat_rumble_menu_item_name"),
        ));
        as_menu.add_item(ConfigBasicMenuItem::new_switch::<CombatPopupSettings>(
            localize::mess::get("combat_popup_name"),
        ));
        as_menu.add_item(ConfigBasicMenuItem::new_switch::<CombatUISettings>(
            localize::mess::get("combat_ui_name"),
        ));
        as_menu.add_item(ConfigBasicMenuItem::new_switch::<RingPolishVibrationSetting>(
            localize::mess::get(get_ring_polish_item_key()),
        ));
        as_menu.add_item(ConfigBasicMenuItem::new_switch::<GameModeSettings>(
            Mess::get("MID_GAMESTART_MODE_TITLE"),
        ));
        as_menu.add_item(ConfigBasicMenuItem::new_switch::<SupportOutfitSetting>(
            localize::mess::get("support_outfit_item_name"),
        ));
        as_menu.add_item(ConfigBasicMenuItem::new_switch::<ToggleRenderScaleSetting>(
            localize::mess::get("render_scale_toggle_name"),
        ));
        as_menu.add_item(ConfigBasicMenuItem::new_switch::<RenderScaleSetting>(
            localize::mess::get("render_scale_name"),
        ));
        as_menu.add_item(ConfigBasicMenuItem::new_switch::<LodSetting>(
            localize::mess::get("lod_name"),
        ));

        BasicMenuResult::se_cursor()
    }

    extern "C" fn set_command_text(this: ConfigBasicMenuItem, _method_info: OptionalMethod) {
        this.set_m_command_text(localize::mess::get("submenu_item_commandtext").into());
    }

    extern "C" fn set_help_text(this: ConfigBasicMenuItem, _method_info: OptionalMethod) {
        this.set_m_help_text(localize::mess::get("cobalt_settings_submenu_item_helptext").into());
    }
}
