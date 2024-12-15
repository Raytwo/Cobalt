use unity::prelude::*;

use engage::{
    menu::{
        config::ConfigBasicMenuItem,
        BasicMenu, BasicMenuItem,
    },
    proc::ProcInst,
    titlebar::TitleBar, mess::Mess, gamedata::unit::Unit, unitpool::UnitPool,
    gameuserdata::GameUserData,
};

mod refine;
mod skillinheritance;
use refine::*;
use skillinheritance::*;

#[skyline::hook(offset = 0x24ee3a0)]
pub fn sortietopmenushopsubmenu_createbind_hook(
    parent_menu: &mut BasicMenu<()>,
    parent_menu_item: &mut ConfigBasicMenuItem,
    method_info: OptionalMethod,
) {
    call_original!(parent_menu, parent_menu_item, method_info);

    let config_menu = parent_menu.proc.child.as_mut().unwrap().cast_mut::<BasicMenu<BasicMenuItem>>();
    config_menu.reserved_show_row_num = 4;

    // Forge button
    let class: &mut Il2CppClass = config_menu.full_menu_item_list.items[0].get_class().clone();
    let new_menu_item = il2cpp::instantiate_class::<BasicMenuItem>(class).unwrap();

    new_menu_item
        .get_class_mut()
        .get_virtual_method_mut("ACall")
        .map(|method| method.method_ptr = refine_menu_item_acall as _);

    new_menu_item
        .get_class_mut()
        .get_virtual_method_mut("GetName")
        .map(|method| method.method_ptr = refine_menu_item_getname as _);

    new_menu_item
        .get_class_mut()
        .get_virtual_method_mut("GetHelpText")
        .map(|method| method.method_ptr = refine_menu_item_gethelptext as _);

    new_menu_item
        .get_class_mut()
        .get_virtual_method_mut("BuildAttribute")
        .map(|method| method.method_ptr = refine_menu_item_buildattribute as _);

    config_menu.full_menu_item_list.add(new_menu_item);

    // Skill Inheritance button
    let class: &mut Il2CppClass = config_menu.full_menu_item_list.items[0].get_class().clone();
    let new_menu_item = il2cpp::instantiate_class::<BasicMenuItem>(class).unwrap();

    new_menu_item
        .get_class_mut()
        .get_virtual_method_mut("ACall")
        .map(|method| method.method_ptr = skill_inherit_acall as _);

    new_menu_item
        .get_class_mut()
        .get_virtual_method_mut("GetName")
        .map(|method| method.method_ptr = skill_inherit_getname as _);

    new_menu_item
        .get_class_mut()
        .get_virtual_method_mut("GetHelpText")
        .map(|method| method.method_ptr = skill_inherit_gethelptext as _);

    config_menu.full_menu_item_list.add(new_menu_item);
}

#[skyline::hook(offset = 0x1d78760)]
extern "C" fn shopmenuitem_getname_hook(_this: &mut BasicMenuItem, _method_info: OptionalMethod) -> &'static Il2CppString {
    Mess::get("MID_SAVEDATA_SEQ_HUB")
}

extern "C" fn open_anime_all_ondispose(this: &mut ProcInst, _method_info: OptionalMethod) {
    TitleBar::close_header();

    this.parent.as_ref().unwrap().get_class().get_virtual_method("OpenAnimeAll").map(|method| {
        let open_anime_all = unsafe { std::mem::transmute::<_, extern "C" fn(&ProcInst, &MethodInfo)>(method.method_info.method_ptr) };
        open_anime_all(this.parent.as_ref().unwrap(), method.method_info);
    });
}

#[unity::hook("App", "GodRoomUnitSelectMenu", "CreateBind")]
pub fn godroomunitselectmenu_createbind_hook(
    parent: &mut ProcInst,
    decide_handler: &mut (),
    select_unit: &Unit,
    method_info: OptionalMethod,
) -> &'static mut BasicMenu<GodRoomUnitSelectMenuItem> {
    let result = call_original!(parent, decide_handler, select_unit, method_info);
    
    // in somniel, do not add force 3 to list 
    if GameUserData::get_sequence() == 4 { return result; } 

    // Get the list of undeployed units
    let force = UnitPool::get_force(3);

    // Manually add them to the menu
    force.iter().for_each(|unit| {
        let new_entry: &mut GodRoomUnitSelectMenuItem = Il2CppObject::from_class(result.full_menu_item_list[0].get_class()).unwrap();

        new_entry.ctor(result.full_menu_item_list.len(), unit, decide_handler);

        result.full_menu_item_list.add(new_entry);
    });

    result
}