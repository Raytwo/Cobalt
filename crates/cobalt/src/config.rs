use engage::app::basicmenu::BasicMenu;
use engage::app::procinst::{IProcInstMethods, ProcInst};
use engage::root::configbasicmenuitem::ConfigBasicMenuItem;
use unity::{Cast, OptionalMethod};

use engage::{BasicMenuExt, ConfigBasicMenuItemExt};

pub mod cobalt;
pub mod combatpopup;
pub mod combatui;
pub mod combatvibration;
pub mod gamemode;
pub mod plugins;
pub mod ringpolishrumble;
pub mod supportoutfit;

use cobalt::*;
use plugins::*;

#[no_mangle]
pub extern "C" fn cobapi_register_configmenuitem_cb(callback: ConfigMenuItemRegistrationCallback) {
    println!("CobAPI received a ConfigMenuItem Registration callback");

    let mut pending_calls = CONFIGMENUITEM_CB.lock().unwrap();
    pending_calls.push(callback);
}

#[unity::hook("", "ConfigMenu", "CreateBind")]
pub fn configmenu_createbind_hook(proc: ProcInst, method_info: OptionalMethod) -> unity::IlInstance {
    let res = call_original!(proc, method_info);

    let child = proc.get_child();

    if !child.is_null() {
        let menu = unsafe { child.cast::<BasicMenu>() };

        let cobalt_settings = ConfigBasicMenuItem::new_command::<CobaltSubmenu>(
            localize::mess::get("cobalt_settings_submenu_item_name"),
        );

        menu.add_item(cobalt_settings);

        let has_plugins = !CONFIGMENUITEM_CB.lock().unwrap().is_empty();
        if has_plugins {
            let plugin_settings = ConfigBasicMenuItem::new_command::<PluginSubmenu>(
                localize::mess::get("plugin_settings_submenu_item_name"),
            );

            menu.add_item(plugin_settings);
        }
    }

    res
}
