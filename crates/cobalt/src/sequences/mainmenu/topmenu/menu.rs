use engage::{
    menu::{BasicMenu, BasicMenuItem, BasicMenuResult},
    proc::ProcInst,
};
use unity::prelude::*;

use crate::sequences::mainmenu::MainMenuSequence;

#[skyline::hook(offset = 0x1b69130)]
pub fn createmenubind_hook(proc: &mut ProcInst, menu_content: *const u8, method_info: OptionalMethod) {
    call_original!(proc, menu_content, method_info);

    proc.child.as_mut().unwrap()
        .get_class_mut()
        .get_virtual_method_mut("GetShowRowMax")
        .map(|method| method.method_ptr = get_show_row_max as _)
        .unwrap();

    let config_menu = proc.child.as_mut().unwrap().cast_mut::<BasicMenu<BasicMenuItem>>();
    config_menu.reserved_show_row_num = 4;

    let cobalt_button = Il2CppObject::<BasicMenuItem>::from_class(config_menu.full_menu_item_list.items[0].get_class().clone()).unwrap();


    cobalt_button
        .get_class_mut()
        .get_virtual_method_mut("ACall")
        .map(|method| method.method_ptr = mod_menu_item_acall as _).unwrap();

    cobalt_button
        .get_class_mut()
        .get_virtual_method_mut("GetName")
        .map(|method| method.method_ptr = mod_menu_item_getname as _).unwrap();

    config_menu.full_menu_item_list.add(cobalt_button);
}

extern "C" fn mod_menu_item_getname(_this: &mut BasicMenuItem, _method_info: OptionalMethod) -> &'static Il2CppString {
    "Cobalt".into()
}

extern "C" fn mod_menu_item_acall(_this: &mut BasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
    let main_menu_seq = MainMenuSequence::get_mut();

    println!("ModMenuItem::ACall");

    main_menu_seq.next_sequence = 31; // Made up Cobalt label

    BasicMenuResult::se_decide().with_close_this(true)
}

extern "C" fn get_show_row_max(_this: &mut BasicMenuItem, _method_info: OptionalMethod) -> i32 {
    4
}
