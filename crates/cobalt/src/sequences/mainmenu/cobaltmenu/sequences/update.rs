use engage::app::basicmenuitem::{BasicMenuItem, IBasicMenuItemMethods};
use engage::app::yesnodialog::YesNoDialog;
use unity::{Il2CppString, OptionalMethod};

use engage::{
    BasicMenuItemAttribute, BasicMenuItemMethods, BasicMenuResult, YesNoDialogExt,
};
use crate::updater::UpdaterDialogChoice;

pub struct UpdateAvailableMenuItem;

impl BasicMenuItemMethods for UpdateAvailableMenuItem {
    extern "C" fn get_name(_this: BasicMenuItem, _method_info: OptionalMethod) -> Il2CppString {
        localize::mess::get("update_available_name").into()
    }

    extern "C" fn a_call(this: BasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        YesNoDialog::bind_with::<UpdaterDialogChoice>(
            this.get_menu(),
            localize::mess::get("update_found_message"),
            localize::mess::get("update_later"),
            localize::mess::get("update_accept"),
        );

        BasicMenuResult::se_decide()
    }

    extern "C" fn build_attribute(
        _this: BasicMenuItem,
        _method_info: OptionalMethod,
    ) -> BasicMenuItemAttribute {
        BasicMenuItemAttribute::enable()
    }
}
