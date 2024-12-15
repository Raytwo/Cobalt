use unity::prelude::*;
use engage::{
    proc::Bindable,
    menu::{BasicMenuItem, BasicMenuResult, BasicMenu},
    sequence::hubrefineshopsequence::HubRefineShopSequence,
    gamedata::{ HubFacilityData, Gamedata },
};

use super::open_anime_all_ondispose;

pub extern "C" fn refine_menu_item_acall(this: &mut BasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
    if !this.is_attribute_disable() {
        this.menu.get_class().get_virtual_method("CloseAnimeAll").map(|method| {
            let close_anime_all =
                unsafe { std::mem::transmute::<_, extern "C" fn(&BasicMenu<BasicMenuItem>, &MethodInfo)>(method.method_info.method_ptr) };
            close_anime_all(this.menu, method.method_info);
        });

        let proc = HubRefineShopSequence::new();

        proc
            .get_class_mut()
            .get_virtual_method_mut("OnDispose")
            .map(|method| method.method_ptr = open_anime_all_ondispose as _)
            .unwrap();

        let descs = proc.create_desc();
        proc.create_bind(this.menu, descs, "HubRefineShopSequence");

        BasicMenuResult::se_decide()
    } else {
        BasicMenuResult::se_miss()
    }
}

pub extern "C" fn refine_menu_item_getname(_this: &mut BasicMenuItem, _method_info: OptionalMethod) -> &'static Il2CppString {
    localize::mess::get("refine_menu_item_name").into()
}

pub extern "C" fn refine_menu_item_gethelptext(_this: &mut BasicMenuItem, _method_info: OptionalMethod) -> &'static Il2CppString {
    localize::mess::get("refine_menu_item_helptext").into()
}

pub extern "C" fn refine_menu_item_buildattribute(_this: &mut BasicMenuItem, _method_info: OptionalMethod) -> i32 {
    let refine_data = HubFacilityData::get("AID_錬成屋").unwrap();

    if refine_data.is_complete() {
        1 // Enabled
    } else {
        2 // Disabled
    }
}

