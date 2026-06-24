use std::sync::OnceLock;

use engage::app::basicdialogitemno::BasicDialogItemNo;
use engage::app::gameuserdata::{GameUserData, GameUserData_Sequences, IGameUserDataMethods};
use engage::app::mainmenusequence::MainMenuSequence;
use engage::app::singletonclass_1::ISingletonClass_1Methods;
use engage::app::singletonprocinst_1::ISingletonProcInst_1Methods;
use engage::system::action::Action;
use engage::system::object::Object;
use unity::prelude::*;
use unity::{FromIlInstance, IlInstance, IntPtr, OptionalMethod};

use engage::{basic_menu_confirm, BasicMenuResult, TwoChoiceDialogMethods};

use crate::api::events::{Event, SystemEvent};

pub static UPDATE: OnceLock<updater::AvailableUpdate> = OnceLock::new();

pub fn update_available() -> bool {
    UPDATE.get().is_some()
}

#[unity::callback]
extern "C" fn apply_update() {
    if MainMenuSequence::get_instance().is_null() || GameUserData::get_instance().get_sequence() != GameUserData_Sequences::none() {
        return;
    }
    let Some(update) = UPDATE.get() else { return };
    if let Err(e) = updater::apply(update) {
        println!("Failed to apply update: {e}");
    }
}

pub struct UpdaterDialogChoice;

impl TwoChoiceDialogMethods for UpdaterDialogChoice {
    extern "C" fn on_second_choice(_this: BasicDialogItemNo, _method_info: OptionalMethod) -> BasicMenuResult {
        apply_update();
        BasicMenuResult::se_decide()
    }
}

pub extern "C" fn update_check(target: MainMenuSequence, _method_info: OptionalMethod) {
    static LOCK: OnceLock<()> = OnceLock::new();

    if LOCK.get().is_none() && update_available() {
        let handler = Action::new(
            <Object as FromIlInstance>::from_il_instance(IlInstance::null()),
            IntPtr(apply_update_method_info() as *const _ as *mut ()),
        );

        let mms_proc = engage::app::procinst::ProcInst::from_il_instance(target.as_instance());
        basic_menu_confirm(
            mms_proc,
            localize::mess::get("update_found_message"),
            localize::mess::get("update_accept"),
            localize::mess::get("update_later"),
            handler,
        );
    }

    LOCK.get_or_init(|| ());
}

pub extern "C" fn catalog_mount_update_check(evt: &Event<SystemEvent>) {
    if let Event::Args(SystemEvent::GamedataLoaded) = evt {
        let _ = std::thread::Builder::new()
            .stack_size(0x10000)
            .spawn(update_thread)
            .unwrap();
    }
}

fn update_thread() {
    match updater::check(crate::utils::env::get_cobalt_version()) {
        Ok(Some(update)) => {
            let _ = UPDATE.set(update);
        },
        Ok(None) => {},
        Err(e) => println!("Update check failed: {e}"),
    }
}
