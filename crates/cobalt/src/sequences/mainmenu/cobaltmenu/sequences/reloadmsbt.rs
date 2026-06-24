use engage::app::basicmenuitem::BasicMenuItem;
use engage::app::language::Language;
use unity::{Il2CppString, OptionalMethod};

use engage::{BasicMenuItemAttribute, BasicMenuItemMethods, BasicMenuResult};

pub struct ReloadMsbtMenuItem;

impl BasicMenuItemMethods for ReloadMsbtMenuItem {
    extern "C" fn get_name(_this: BasicMenuItem, _method_info: OptionalMethod) -> Il2CppString {
        localize::mess::get("reload_message").into()
    }

    extern "C" fn a_call(_this: BasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        Language::reflect_setting();
        BasicMenuResult::se_decide()
    }

    extern "C" fn build_attribute(
        _this: BasicMenuItem,
        _method_info: OptionalMethod,
    ) -> BasicMenuItemAttribute {
        BasicMenuItemAttribute::enable()
    }
}
