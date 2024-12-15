use unity::prelude::*;

use engage::{
    menu::{
        config::ConfigBasicMenuItem, ConfigMenu
    },
    proc::ProcInst, 
};

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

#[skyline::hook(offset = 0x2538a10)]
pub fn configmenu_createbind_hook(proc: &mut ProcInst, method_info: OptionalMethod) -> &'static *const u8 {
    let res = call_original!(proc, method_info);

    let config_menu = proc.child.as_mut().unwrap().cast_mut::<ConfigMenu<ConfigBasicMenuItem>>();

    let cobalt_settings = ConfigBasicMenuItem::new_command::<CobaltSubmenu>(localize::mess::get("cobalt_settings_submenu_item_name"));
    
    config_menu.add_item(cobalt_settings);
    
    if !CONFIGMENUITEM_CB.lock().unwrap().is_empty() {
        let plugin_settings = ConfigBasicMenuItem::new_command::<PluginSubmenu>(localize::mess::get("plugin_settings_submenu_item_name"));
        config_menu.add_item(plugin_settings);
    }

    res
}

pub extern "C" fn open_anime_all_ondispose(this: &mut ProcInst, _method_info: OptionalMethod) {
    this.parent.as_ref().unwrap().get_class().get_virtual_method("OpenAnimeAll").map(|method| {
        let open_anime_all = unsafe { std::mem::transmute::<_, extern "C" fn(&ProcInst, &MethodInfo)>(method.method_info.method_ptr) };
        open_anime_all(this.parent.as_ref().unwrap(), method.method_info);
    });
}