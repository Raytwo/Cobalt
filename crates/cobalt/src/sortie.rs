use engage::app::basicmenu::{BasicMenu, IBasicMenu};
use engage::app::basicmenuitem::BasicMenuItem;
use engage::app::procinst::{IProcInstMethods, ProcInst};
use engage::app::unit::Unit;
use engage::app::unitpool::UnitPool;
use engage::{ForceExt, TitleBar};
use engage::app::gameuserdata::{GameUserData, GameUserData_Sequences, IGameUserDataMethods};
use engage::app::singletonclass_1::ISingletonClass_1Methods;
use unity::prelude::*;
use engage::{BasicMenuExt, List_1Ext, Mess};
use unity::{Cast, FromIlInstance};

mod refine;
mod skillinheritance;

pub use refine::RefineMenuItem;
pub use skillinheritance::SkillInheritMenuItem;

#[skyline::hook(offset = 0x24ee3a0)]
pub fn sortietopmenushopsubmenu_createbind_hook(
    parent_menu: ProcInst,
    parent_menu_item: IlInstance,
    method_info: OptionalMethod,
) {
    call_original!(parent_menu, parent_menu_item, method_info);

    let child = parent_menu.get_child();
    if child.is_null() {
        return;
    }
    let config_menu = <BasicMenu as FromIlInstance>::from_il_instance(child.as_instance());
    config_menu.set_m_reserved_show_row_num(4);

    config_menu.add_item(refine::new_refine_menu_item());
    config_menu.add_item(skillinheritance::new_skill_inherit_menu_item());
}

#[skyline::hook(offset = 0x1d78760)]
extern "C" fn shopmenuitem_getname_hook(_this: BasicMenuItem, _method_info: OptionalMethod) -> Il2CppString {
    Mess::get("MID_SAVEDATA_SEQ_HUB")
}

pub(crate) extern "C" fn open_anime_all_ondispose(this: ProcInst, _method_info: OptionalMethod) {
    TitleBar::close_header();

    let parent = this.get_super();
    if parent.is_null() {
        return;
    }
    if let Some(slot) = parent.get_class().raw().get_virtual_method("OpenAnimeAll") {
        let open_anime_all: extern "C" fn(ProcInst, &'static unity::MethodInfo) =
            unsafe { std::mem::transmute(slot.method_ptr) };
        open_anime_all(parent, slot.method_info);
    }
}

#[unity::hook("App", "GodRoomUnitSelectMenu", "CreateBind")]
pub fn godroomunitselectmenu_createbind_hook(
    parent: ProcInst,
    decide_handler: IlInstance,
    select_unit: Unit,
    method_info: OptionalMethod,
) -> BasicMenu {
    let result = call_original!(parent, decide_handler, select_unit, method_info);

    if GameUserData::get_instance().get_sequence() == GameUserData_Sequences::hub() {
        return result;
    }

    let force = UnitPool::get_force(3);
    let menu_list = result.m_full_menu_item_list();
    let template_class = menu_list
        .iter()
        .next()
        .expect("GodRoomUnitSelectMenu had no rows at CreateBind time")
        .get_class();
    force.iter().for_each(|unit| {
        let new_entry = <BasicMenuItem as FromIlInstance>::instantiate_with_class(template_class)
            .expect("GodRoomUnitSelectMenuItem instantiation failed");
        skillinheritance::godroomunitselectmenuitem_ctor(
            new_entry,
            menu_list.count() as usize,
            unit,
            decide_handler,
        );
        result.add_item(new_entry);
    });

    result
}
