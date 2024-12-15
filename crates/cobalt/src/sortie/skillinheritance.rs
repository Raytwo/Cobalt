use engage::{mess::Mess, menu::{BasicMenuResult, BasicMenuItem, BasicMenu, BasicMenuFields, BasicMenuMethods}, sequence::skillinheritancesequence::SkillInheritanceSequence, proc::Bindable, gamedata::unit::Unit};
use unity::prelude::*;

use crate::sortie::open_anime_all_ondispose;

#[unity::class("App", "GodRoomUnitSelectMenu")]
pub struct GodRoomUnitSelectMenu<T: 'static> {
    pub base: BasicMenuFields<T>,
    // ...
}

impl<T> BasicMenuMethods for GodRoomUnitSelectMenu<T> { }

#[unity::class("", "GodRoomUnitSelectMenu.GodRoomUnitSelectMenuItem")]
pub struct GodRoomUnitSelectMenuItem {
    pub menu: &'static mut BasicMenu<BasicMenuItem>,
    menu_item_content: *const u8,
    name: &'static Il2CppString,
    pad: i32,
    full_index: i32,
    attribute: i32,
    cursor_color: unity::engine::Color,
    active_text_color: unity::engine::Color,
    inactive_text_color: unity::engine::Color,
    pub index: i32,
    pub unit: &'static Unit,
    pub decide_handler: &'static (),
}

impl GodRoomUnitSelectMenuItem {
    pub fn ctor(&self, index: usize, unit: &'static Unit, decide_handler: &()){
        self.get_class().get_methods().iter().find(|method| method.get_name() == Some(String::from(".ctor"))).map(|method| {
            let ctor = unsafe { std::mem::transmute::<_, extern "C" fn(&GodRoomUnitSelectMenuItem, i32, &Unit, &(), &MethodInfo)>(method.method_ptr) };
            ctor(&self, index as i32, unit, &decide_handler, method);
        }).unwrap();
    }
}

#[repr(C)]
pub struct GodRoomUnitSelectMenuStaticFields {
    force_mask: u32,
    scroll_index: i32,
}

pub extern "C" fn skill_inherit_acall(this: &'static mut BasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
    // We instantiate it in case the player didn't enter the emblem room this session, so the static fields are initialized
    let unit_select_menu = GodRoomUnitSelectMenu::<()>::instantiate().unwrap().get_class_mut().get_static_fields_mut::<GodRoomUnitSelectMenuStaticFields>();

    // Have the skill inheritance menu display the players instead of undeployed units
    unit_select_menu.force_mask = 0x9;

    this.menu.get_class().get_virtual_method("CloseAnimeAll").map(|method| {
        let close_anime_all =
            unsafe { std::mem::transmute::<_, extern "C" fn(&BasicMenu<BasicMenuItem>, &MethodInfo)>(method.method_info.method_ptr) };
        close_anime_all(this.menu, method.method_info);
    });
    
    let sequence = SkillInheritanceSequence::new();

    sequence
        .get_class_mut()
        .get_virtual_method_mut("OnDispose")
        .map(|method| method.method_ptr = open_anime_all_ondispose as _)
        .unwrap();
    
    let descs = sequence.create_desc();
    sequence.create_bind(this.menu, descs, "SkillInheritanceSequence");

    BasicMenuResult::se_decide()
}

pub extern "C" fn skill_inherit_getname(_this: &mut BasicMenuItem, _method_info: OptionalMethod) -> &'static Il2CppString {
    Mess::get("MID_Hub_Skill_Inheritance")
}

pub extern "C" fn skill_inherit_gethelptext(_this: &mut BasicMenuItem, _method_info: OptionalMethod) -> &'static Il2CppString {
    Mess::get("MID_Hub_Inheritance_Skill_HELP")
}