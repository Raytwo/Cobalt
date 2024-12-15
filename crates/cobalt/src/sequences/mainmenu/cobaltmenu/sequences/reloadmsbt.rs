use engage::{menu::{BasicMenuItemAttribute, BasicMenuItem, BasicMenuResult, BasicMenuItemMethods}, language::Language};
use unity::prelude::*;

pub struct ReloadMsbtMenuItem;

impl BasicMenuItemMethods for ReloadMsbtMenuItem {
    extern "C" fn get_name(_this: &mut BasicMenuItem, _method_info: OptionalMethod) -> &'static Il2CppString {
        localize::mess::get("reload_message").into()
    }

    extern "C" fn a_call(_this: &'static mut BasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        Language::reflect_setting();
        BasicMenuResult::se_decide()
    }

    extern "C" fn build_attributes(_this: &mut BasicMenuItem, _method_info: OptionalMethod) -> BasicMenuItemAttribute {
        BasicMenuItemAttribute::Enable
    }
}
