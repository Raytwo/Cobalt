use engage::app::basicmenuitem::{BasicMenuItem, IBasicMenuItemMethods};
use engage::app::hubfacilitydata::{HubFacilityData, IHubFacilityDataMethods};
use engage::app::hubrefineshopsequence::{HubRefineShopSequence, IHubRefineShopSequenceMethods};
use engage::app::procinst::{IProcInstMethods, ProcInst};
use engage::app::structdata_1::IStructData_1Methods;
use unity::prelude::*;
use unity::Cast;

use engage::{BasicMenuExt, BasicMenuItemAttribute, BasicMenuItemExt, BasicMenuItemMethods, BasicMenuResult};
use super::open_anime_all_ondispose;

pub struct RefineMenuItem;

impl BasicMenuItemMethods for RefineMenuItem {
    extern "C" fn get_name(_this: BasicMenuItem, _method_info: OptionalMethod) -> Il2CppString {
        localize::mess::get("refine_menu_item_name").into()
    }

    extern "C" fn a_call(this: BasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let menu = this.get_menu();
        BasicMenuExt::close_anime_all(menu);

        let seq = HubRefineShopSequence::new();

        let descs = seq.create_desc();
        let menu_proc: ProcInst = menu.into();
        let name: Il2CppString = "HubRefineShopSequence".into();
        let seq_proc: ProcInst = seq.into();
        seq_proc.create_bind(menu_proc, descs, name);

        seq.override_class()
            .override_virtual_method("OnDispose", unity::method_info!(open_anime_all_ondispose, 0));

        BasicMenuResult::se_decide()
    }

    extern "C" fn build_attribute(
        _this: BasicMenuItem,
        _method_info: OptionalMethod,
    ) -> BasicMenuItemAttribute {
        let refine_data = HubFacilityData::get("AID_錬成屋".into());
        if refine_data.is_null() || !refine_data.is_complete() {
            BasicMenuItemAttribute::disable()
        } else {
            BasicMenuItemAttribute::enable()
        }
    }
}

extern "C" fn refine_menu_item_gethelptext(
    _this: BasicMenuItem,
    _method_info: OptionalMethod,
) -> Il2CppString {
    localize::mess::get("refine_menu_item_helptext").into()
}

pub fn new_refine_menu_item() -> BasicMenuItem {
    let item = BasicMenuItem::new_impl::<RefineMenuItem>();
    item.get_class()
        .override_virtual_method("GetHelpText", unity::method_info!(refine_menu_item_gethelptext, 0));
    item
}

