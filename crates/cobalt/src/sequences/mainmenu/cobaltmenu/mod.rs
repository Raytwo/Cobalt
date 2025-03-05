use unity::{prelude::*, system::List};

use engage::{
    menu::{BasicMenu, BasicMenuItem, BasicMenuMethods, MenuSequence},
    proc::{desc::ProcDesc, Bindable, ProcBoolMethod, ProcInst, ProcVoidMethod},
    titlebar::TitleBar,
};

pub mod sequences;

use sequences::{
    reloadmsbt::*,
    reloadxml::*,
    settings::*,
    update::*
};

pub struct CobaltMenuSequence;

#[repr(i32)]
enum CobaltMenuSequenceLabel {
    Start = 1,
    Exit,
}

impl MenuSequence for CobaltMenuSequence {
    fn get_proc_desc(this: &'static ProcInst) -> Vec<&'static mut ProcDesc> {
        // Create and display the menu

        vec![
            ProcDesc::label(CobaltMenuSequenceLabel::Start as _),
            ProcDesc::call(ProcVoidMethod::new(this, Self::load_resources)),
            ProcDesc::wait_while_true(ProcBoolMethod::new(this, Self::is_loading_resources)),
            ProcDesc::call(ProcVoidMethod::new(this, Self::create_menu_bind)),
            ProcDesc::label(CobaltMenuSequenceLabel::Exit as _),
            ProcDesc::call(ProcVoidMethod::new(this, Self::exit)),
            ProcDesc::end(),
        ]
    }

    fn proc_name() -> &'static str {
        "CobaltMenuSequence"
    }
}

impl CobaltMenuSequence {
    pub extern "C" fn load_resources(_parent: &mut impl Bindable, _method_info: OptionalMethod) {
        // Load the prefab for the Shop UI
        unsafe { engage::menu::content::shoptopmenu::shop_top_menu_content_load_prefab_async(None) };
    }

    pub extern "C" fn exit(_parent: &mut impl Bindable, _method_info: OptionalMethod) {
        TitleBar::close_header();

        unsafe {
            // Unload the prefab for the Shop UI
            engage::menu::content::shoptopmenu::shop_top_menu_content_unload_prefab(None);
        }
    }

    pub extern "C" fn is_loading_resources(_parent: &mut impl Bindable, _method_info: OptionalMethod) -> bool {
        // This method will be pooled continuously until all the resources are loaded
        unsafe { engage::resourcemanager::is_loading(None) }
    }

    pub extern "C" fn create_menu_bind(parent: &'static mut impl Bindable, _method_info: OptionalMethod) {
        println!("CobaltMenuSequence::CreateMenuBind");

        let menu_content = unsafe { engage::menu::content::shoptopmenu::shop_top_menu_content_create(None) }.unwrap();

        // Create a List<BasicMenuItem> for the BasicMenu
        let menu_item_list_class = get_generic_class!(SystemList<BasicMenuItem>).unwrap();
        let menu_item_list = il2cpp::instantiate_class::<List<BasicMenuItem>>(&menu_item_list_class).unwrap();
        // Create a item list with a capacity of 1
        menu_item_list.items = Il2CppArray::new(3).unwrap();

        // Create and add our button to the item list
        let menu_item = BasicMenuItem::new_impl::<ReloadXmlMenuItem>();
        menu_item_list.add(menu_item);
        let menu_item = BasicMenuItem::new_impl::<ReloadMsbtMenuItem>();
        menu_item_list.add(menu_item);
        let menu_item = BasicMenuItem::new_impl::<GlobalConfigMenuItem>();
        menu_item_list.add(menu_item);
        
        if crate::updater::UPDATE_AVAILABLE.load(std::sync::atomic::Ordering::Relaxed) {
            let menu_item = BasicMenuItem::new_impl::<UpdateAvailableMenuItem>();
            menu_item_list.add(menu_item);
        }

        // Create a BasicMenu and fill it with our item list
        let basic_menu = BasicMenu::new(menu_item_list, menu_content);
        let descs = basic_menu.create_default_desc();
        
        basic_menu.create_bind(parent, descs, "CobaltMenu");
        basic_menu.bind_parent_menu();

        TitleBar::open_header("Cobalt", localize::mess::get("cobalt_main_menu_title_bar_text"), "");

        // Don't let the babeys enter the menu
        // GameMessage::create_key_wait(parent, "Nothing to see here.");
    }
}

#[repr(C)]
#[unity::class("System.Collections.Generic", "List`1")]
pub struct SystemList {}