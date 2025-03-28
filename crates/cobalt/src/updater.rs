use unity::prelude::*;
use std::sync::{atomic::AtomicBool, OnceLock};

use engage::{
    dialog::yesno::{BasicDialogItemNo, TwoChoiceDialogMethods, YesNoDialog}, menu::BasicMenuResult, sequence::mainmenusequence::MainMenuSequence
};

use crate::api::events::SystemEvent;

pub static UPDATE_AVAILABLE: AtomicBool = AtomicBool::new(false);


pub struct UpdaterDialogChoice;

impl TwoChoiceDialogMethods for UpdaterDialogChoice {
    extern "C" fn on_second_choice(_this: &mut BasicDialogItemNo, _method_info: OptionalMethod) -> BasicMenuResult {
        updater::check_for_updates(crate::utils::env::get_cobalt_version(), |_, _, _| true);
        // Supposedly there is nothing to return here since the game will reboot but let's accomodate the compiler
        BasicMenuResult::se_decide()
    }
}

// #[unity::class("Ray", "UnitItem")]
// pub struct RayUnitItem { }

pub extern "C" fn update_check(target: &'static mut MainMenuSequence, _method_info: OptionalMethod) {
    static LOCK: OnceLock<()> = OnceLock::new();

    // Return immediately if we've already checked for this play session.
    // FIXME: This check is currently bad and very slow and forces us to return if we want to update or not immediately, this'll need a rewrite.
    if LOCK.get().is_none() {
        if UPDATE_AVAILABLE.load(std::sync::atomic::Ordering::Relaxed) {
            YesNoDialog::bind::<UpdaterDialogChoice>(target, localize::mess::get("update_found_message"), localize::mess::get("update_later"), localize::mess::get("update_accept"));
        }
    }

    LOCK.get_or_init(|| ());
}

pub extern "C" fn catalog_mount_update_check(evt: &crate::api::events::Event<SystemEvent>) {
    if let crate::api::events::Event::Args(ev) = evt {
        if let SystemEvent::CatalogLoaded = ev {
            let _updater = std::thread::Builder::new()
                .stack_size(0x10000)
                .spawn(|| {
                    update_thread();
                })
                .unwrap();
        }
    }
}

fn update_thread() {
    updater::check_for_updates(crate::utils::env::get_cobalt_version(), |_, _, _| {
        // Set the global bool so we know an update is available for later.
        UPDATE_AVAILABLE.store(true, std::sync::atomic::Ordering::Relaxed);
        // We don't want to update yet, just know if an update is available
        false
    });
}