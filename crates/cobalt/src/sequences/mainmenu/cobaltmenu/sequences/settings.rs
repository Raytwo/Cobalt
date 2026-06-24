use engage::app::basicmenu::{BasicMenu, IBasicMenu};
use engage::app::basicmenuitem::BasicMenuItem;
use engage::app::gameuserdata::{GameUserData, GameUserData_Sequences, IGameUserDataMethods};
use engage::app::proc::Proc;
use engage::app::procinst::{IProcInst, IProcInstMethods, ProcInst};
use engage::app::procvoidmethod::ProcVoidMethod;
use engage::app::singletonclass_1::ISingletonClass_1Methods;
use engage::root::configbasicmenuitem::ConfigBasicMenuItem;
use engage::root::configmenu::ConfigMenu;
use engage::root::configsequence::ConfigSequence;
use engage::system::collections::generic::list_1::IList_1Methods as IListMethods;
use unity::{Cast, Il2CppString, IlInstance, OptionalMethod};

use engage::{
    BasicMenuExt, BasicMenuItemAttribute, BasicMenuItemMethods, BasicMenuResult,
    ConfigBasicMenuItemExt, ProcExt, ProcVoidMethodExt, TitleBar,
};

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
    extern "C" fn get_name(_this: BasicMenuItem, _method_info: OptionalMethod) -> Il2CppString {
        localize::mess::get("global_settings_title").into()
    }

    extern "C" fn a_call(this: BasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        use engage::app::basicmenuitem::IBasicMenuItemMethods;
        let parent_menu = this.get_menu();
        BasicMenuExt::close_anime_all(parent_menu);
        TitleBar::close_header();

        let parent_menu_proc: ProcInst = parent_menu.into();
        ConfigSequence::create_bind(parent_menu_proc);

        let child = parent_menu_proc.get_child();
        if child.is_null() {
            return BasicMenuResult::se_decide();
        }

        let child_proc = unsafe { child.cast::<ProcInst>() };
        let descs = child_proc.m_descs();
        let custom = ProcVoidMethod::from_fn(child_proc.into(), globalconfigsequence_createconfigmenu)
            .expect("GlobalConfigMenuItem: ProcVoidMethod allocation failed");
        descs.set(4, Proc::call_method(custom));

        child_proc.override_class().override_virtual_method(
            "OnDispose",
            unity::method_info!(open_anime_all_ondispose, 0),
        );

        BasicMenuResult::se_decide()
    }

    extern "C" fn build_attribute(
        _this: BasicMenuItem,
        _method_info: OptionalMethod,
    ) -> BasicMenuItemAttribute {
        BasicMenuItemAttribute::enable()
    }
}

pub extern "C" fn open_anime_all_ondispose(this: IlInstance, _method_info: OptionalMethod) {
    if GameUserData::get_instance().get_sequence() == GameUserData_Sequences::none() {
        TitleBar::open_header(
            "Cobalt",
            localize::mess::get("cobalt_main_menu_title_bar_text"),
            "",
        );
    }
    let proc = <ProcInst as unity::FromIlInstance>::from_il_instance(this);
    let parent = proc.get_super();
    if parent.is_null() {
        return;
    }
    if let Some(slot) = parent.get_class().raw().get_virtual_method("OpenAnimeAll") {
        let f: extern "C" fn(ProcInst, &'static unity::MethodInfo) =
            unsafe { std::mem::transmute(slot.method_ptr) };
        f(parent, slot.method_info);
    }
}

pub extern "C" fn globalconfigsequence_createconfigmenu(
    seq_proc: ProcInst,
    _method_info: OptionalMethod,
) {
    ConfigMenu::create_bind(seq_proc);

    let child = seq_proc.get_child();
    if child.is_null() {
        return;
    }
    let as_menu = unsafe { child.cast::<BasicMenu>() };
    as_menu.m_full_menu_item_list().clear();

    if !GLOBAL_CONFIGMENUITEM_CB.lock().unwrap().is_empty() {
        let plugin_settings = ConfigBasicMenuItem::new_command::<GlobalPluginSubmenu>(
            localize::mess::get("plugin_settings_submenu_item_name"),
        );
        as_menu.add_item(plugin_settings);
    }

    let skip_opening = ConfigBasicMenuItem::new_switch::<SkipOpeningSetting>(
        localize::mess::get("skip_opening_menu_item_name"),
    );
    as_menu.add_item(skip_opening);
}
