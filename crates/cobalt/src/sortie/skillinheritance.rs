use engage::app::basicmenuitem::{BasicMenuItem, IBasicMenuItemMethods};
use engage::app::godroomunitselectmenu::GodRoomUnitSelectMenu;
use engage::app::procinst::{IProcInstMethods, ProcInst};
use engage::app::skillinheritancesequence::{
    ISkillInheritanceSequenceMethods, SkillInheritanceSequence,
};
use engage::app::unit::Unit;
use unity::prelude::*;
use unity::{Cast, MethodInfo};

use engage::{BasicMenuExt, BasicMenuItemExt, BasicMenuItemMethods, BasicMenuResult, Mess};
use super::open_anime_all_ondispose;

pub fn godroomunitselectmenuitem_ctor(
    item: BasicMenuItem,
    index: usize,
    unit: Unit,
    decide_handler: IlInstance,
) {
    let class = item.get_class();
    let method = class
        .raw()
        .get_method_from_name(".ctor", 3)
        .expect("GodRoomUnitSelectMenuItem::.ctor missing from runtime class");
    let ctor: extern "C" fn(BasicMenuItem, i32, Unit, IlInstance, &'static MethodInfo) =
        unsafe { std::mem::transmute(method.method_ptr) };
    ctor(item, index as i32, unit, decide_handler, method);
}

pub struct SkillInheritMenuItem;

impl BasicMenuItemMethods for SkillInheritMenuItem {
    extern "C" fn get_name(_this: BasicMenuItem, _method_info: OptionalMethod) -> Il2CppString {
        Mess::get("MID_Hub_Skill_Inheritance")
    }

    extern "C" fn a_call(this: BasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let _ = <GodRoomUnitSelectMenu as unity::FromIlInstance>::instantiate()
            .expect("GodRoomUnitSelectMenu::instantiate (priming cctor)");
        let class = <GodRoomUnitSelectMenu as unity::ClassIdentity>::class();
        unity::static_field_set_value_at_offset::<u32>(class, 0, 0x9);

        let menu = this.get_menu();
        BasicMenuExt::close_anime_all(menu);

        let seq = SkillInheritanceSequence::new();

        let descs = seq.create_desc();
        let menu_proc: ProcInst = menu.into();
        let name: Il2CppString = "SkillInheritanceSequence".into();
        let seq_proc: ProcInst = seq.into();
        seq_proc.create_bind(menu_proc, descs, name);

        seq.override_class()
            .override_virtual_method("OnDispose", unity::method_info!(open_anime_all_ondispose, 0));

        BasicMenuResult::se_decide()
    }
}

extern "C" fn skill_inherit_gethelptext(
    _this: BasicMenuItem,
    _method_info: OptionalMethod,
) -> Il2CppString {
    Mess::get("MID_Hub_Inheritance_Skill_HELP")
}

pub fn new_skill_inherit_menu_item() -> BasicMenuItem {
    let item = BasicMenuItem::new_impl::<SkillInheritMenuItem>();
    item.get_class()
        .override_virtual_method("GetHelpText", unity::method_info!(skill_inherit_gethelptext, 0));
    item
}
