use unity::prelude::*
;
use std::sync::Mutex;

use engage::{
    menu::{
        BasicMenu, BasicMenuResult, ConfigMenu,
        config::{
            ConfigBasicMenuItem,
            ConfigBasicMenuItemCommandMethods
        }, 
    },
    pad::Pad,
    util::get_instance
};

use super::open_anime_all_ondispose;

pub type GlobalConfigMenuItemRegistrationCallback = extern "C" fn() -> &'static mut ConfigBasicMenuItem;

pub static GLOBAL_CONFIGMENUITEM_CB: Mutex<Vec<GlobalConfigMenuItemRegistrationCallback>> = Mutex::new(Vec::new());

#[no_mangle]
pub extern "C" fn cobapi_register_global_configmenuitem_cb(callback: GlobalConfigMenuItemRegistrationCallback) {
    println!("CobAPI received a Global ConfigMenuItem Registration callback");

    let mut pending_calls = GLOBAL_CONFIGMENUITEM_CB.lock().unwrap();
    pending_calls.push(callback);
}

pub struct GlobalPluginSubmenu;

impl ConfigBasicMenuItemCommandMethods for GlobalPluginSubmenu {
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let pad_instance = get_instance::<Pad>();

        // Check if A is pressed before executing any of this
        if pad_instance.npad_state.buttons.a() {
            if !pad_instance.old_buttons.a() {
                // Close the original Settings menu temporarily so it doesn't get drawn in the background
                this.menu.close_anime_all();

                // Initialize the menu
                ConfigMenu::create_bind(this.menu);
                
                let config_menu = this.menu.proc.child.as_mut().unwrap().cast_mut::<BasicMenu<ConfigBasicMenuItem>>();

                // Register a OnDispose callback to restore the previous menu
                config_menu
                    .get_class_mut()
                    .get_virtual_method_mut("OnDispose")
                    .map(|method| method.method_ptr = open_anime_all_ondispose as _)
                    .unwrap();

                // Clear the buttons in the List so we can add our own
                config_menu.full_menu_item_list.clear();

                GLOBAL_CONFIGMENUITEM_CB.lock().unwrap().iter().for_each(|cb| {
                    config_menu.full_menu_item_list.add(cb());
                });

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
        this.help_text = localize::mess::get("plugin_settings_submenu_item_helptext").into();
    }
}