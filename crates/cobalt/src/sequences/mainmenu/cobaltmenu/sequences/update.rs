use engage::{dialog::yesno::YesNoDialog, menu::{BasicMenuItem, BasicMenuItemAttribute, BasicMenuItemMethods, BasicMenuResult}};
use unity::prelude::*;

use crate::updater::UpdaterDialogChoice;

pub struct UpdateAvailableMenuItem;

impl BasicMenuItemMethods for UpdateAvailableMenuItem {
    extern "C" fn get_name(_this: &mut BasicMenuItem, _method_info: OptionalMethod) -> &'static Il2CppString {
        localize::mess::get("update_available_name").into()
    }

    extern "C" fn a_call(this: &'static mut BasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        YesNoDialog::bind::<UpdaterDialogChoice>(this.menu, localize::mess::get("update_found_message"), localize::mess::get("update_later"), localize::mess::get("update_accept"));
        
        BasicMenuResult::se_decide()
    }

    extern "C" fn build_attributes(_this: &mut BasicMenuItem, _method_info: OptionalMethod) -> BasicMenuItemAttribute {
        BasicMenuItemAttribute::Enable
    }
}
