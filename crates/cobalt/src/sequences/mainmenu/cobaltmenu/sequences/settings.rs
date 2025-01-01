use engage::{
    menu::{config::ConfigBasicMenuItem, BasicMenuItem, BasicMenuItemAttribute, BasicMenuItemMethods, BasicMenuResult, ConfigMenu},
    proc::{desc::ProcDesc, ProcInst, ProcVoidMethod},
    sequence::configsequence::ConfigSequence,
    titlebar::TitleBar,
};
use unity::prelude::*;

mod opening;
pub mod plugins;
use opening::SkipOpeningSetting;
use plugins::GlobalPluginSubmenu;
pub mod lod;
pub mod render_scale;
pub mod render_scale_toggle;
pub mod util;

use self::plugins::GLOBAL_CONFIGMENUITEM_CB;

pub struct GlobalConfigMenuItem;

impl BasicMenuItemMethods for GlobalConfigMenuItem {
    extern "C" fn get_name(_this: &mut BasicMenuItem, _method_info: OptionalMethod) -> &'static Il2CppString {
        localize::mess::get("global_settings_title").into()
    }

    extern "C" fn a_call(this: &'static mut BasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        // Close the Cobalt menu when entering the settings
        this.menu.close_anime_all();

        // Initialize the menu
        ConfigSequence::create_bind(this.menu);

        // Register a OnDispose callback to restore the previous menu
        this.menu
            .proc
            .child
            .as_mut()
            .unwrap()
            .get_class_mut()
            .get_virtual_method_mut("OnDispose")
            .map(|method| method.method_ptr = open_anime_all_ondispose as _)
            .unwrap();

        let create_global_config_menu = ProcVoidMethod::new(None, globalconfigsequence_createconfigmenu);

        // Replace CreateConfigMenu by our own implementation
        this.menu
            .proc
            .child
            .as_mut()
            .unwrap()
            .get_descs_mut()
            .get_mut(4)
            .map(|call| *call = ProcDesc::call(create_global_config_menu))
            .unwrap();

        BasicMenuResult::se_decide()
    }

    extern "C" fn build_attributes(_this: &mut BasicMenuItem, _method_info: OptionalMethod) -> BasicMenuItemAttribute {
        BasicMenuItemAttribute::Enable
    }
}

pub extern "C" fn open_anime_all_ondispose(this: &mut ProcInst, _method_info: OptionalMethod) {
    // TODO: Make a variant that accepts Mess IDs
    TitleBar::open_header("Cobalt", localize::mess::get("cobalt_main_menu_title_bar_text"), "");

    this.parent
        .as_ref()
        .unwrap()
        .get_class()
        .get_virtual_method("OpenAnimeAll")
        .map(|method| {
            let open_anime_all = unsafe { std::mem::transmute::<_, extern "C" fn(&ProcInst, &MethodInfo)>(method.method_info.method_ptr) };
            open_anime_all(this.parent.as_ref().unwrap(), method.method_info);
        });
}

pub extern "C" fn globalconfigsequence_createconfigmenu(this: &mut ConfigSequence, _method_info: OptionalMethod) {
    // Initialize the menu
    ConfigMenu::create_bind(this);

    let config_menu = this.proc.child.as_mut().unwrap().cast_mut::<ConfigMenu<ConfigBasicMenuItem>>();

    // Clear the buttons in the List so we can add our own
    config_menu.full_menu_item_list.clear();

    if !GLOBAL_CONFIGMENUITEM_CB.lock().unwrap().is_empty() {
        let plugin_settings = ConfigBasicMenuItem::new_command::<GlobalPluginSubmenu>(localize::mess::get("plugin_settings_submenu_item_name"));
        config_menu.add_item(plugin_settings);
    }

    let skip_opening = ConfigBasicMenuItem::new_switch::<SkipOpeningSetting>(localize::mess::get("skip_opening_menu_item_name"));
    config_menu.add_item(skip_opening);
}
