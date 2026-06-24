use engage::app::basicmenu::{BasicMenu, IBasicMenu};
use engage::app::basicmenuitem::BasicMenuItem;
use engage::app::mainmenusequence::{IMainMenuSequence, MainMenuSequence, MainMenuSequence_Label};
use engage::app::procinst::{IProcInstMethods, ProcInst};
use engage::app::singletonprocinst_1::ISingletonProcInst_1Methods;
use unity::prelude::*;
use engage::{
    BasicMenuExt, BasicMenuItemExt, BasicMenuItemMethods, BasicMenuResult, List_1Ext,
};

#[skyline::hook(offset = 0x1b69130)]
pub fn createmenubind_hook(proc: ProcInst, menu_content: IlInstance, method_info: OptionalMethod) {
    call_original!(proc, menu_content, method_info);

    let child = proc.get_child();
    if child.is_null() {
        return;
    }

    let config_menu = <BasicMenu as unity::FromIlInstance>::from_il_instance(child.as_instance());

    let child_class = child.override_class();
    child_class.override_virtual_method("GetShowRowMax", unity::method_info!(get_show_row_max, 0));

    config_menu.set_m_reserved_show_row_num(4);

    let full_list = config_menu.m_full_menu_item_list();
    if full_list.count() == 0 {
        return;
    }
    let template = full_list.get(0);
    let cobalt_button = BasicMenuItem::new_impl_from_template::<CobaltMenuItem>(template);
    config_menu.add_item(cobalt_button);
}

struct CobaltMenuItem;

impl BasicMenuItemMethods for CobaltMenuItem {
    extern "C" fn get_name(_this: BasicMenuItem, _method_info: OptionalMethod) -> Il2CppString {
        "Cobalt".into()
    }

    extern "C" fn a_call(_this: BasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        println!("CobaltMenuItem::ACall");
        MainMenuSequence::get_instance().set_m_next_sequence(MainMenuSequence_Label { value: 31 });
        BasicMenuResult::se_decide().with_close_this(true)
    }
}

extern "C" fn get_show_row_max(_this: BasicMenu, _method_info: OptionalMethod) -> i32 {
    4
}
