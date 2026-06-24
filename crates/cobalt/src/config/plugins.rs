use std::sync::Mutex;

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
};

pub type ConfigMenuItemRegistrationCallback = extern "C" fn() -> ConfigBasicMenuItem;

pub static CONFIGMENUITEM_CB: Mutex<Vec<ConfigMenuItemRegistrationCallback>> = Mutex::new(Vec::new());

pub struct PluginSubmenu;

impl ConfigBasicMenuItemCommandMethods for PluginSubmenu {
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

        let callbacks: Vec<ConfigMenuItemRegistrationCallback> =
            CONFIGMENUITEM_CB.lock().unwrap().clone();
        for cb in &callbacks {
            let item = cb();
            item.override_class();
            as_menu.add_item(item);
        }

        BasicMenuResult::se_cursor()
    }

    extern "C" fn set_command_text(this: ConfigBasicMenuItem, _method_info: OptionalMethod) {
        this.set_m_command_text(localize::mess::get("submenu_item_commandtext").into());
    }

    extern "C" fn set_help_text(this: ConfigBasicMenuItem, _method_info: OptionalMethod) {
        this.set_m_help_text(localize::mess::get("plugin_settings_submenu_item_helptext").into());
    }
}
