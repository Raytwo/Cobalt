#![feature(stmt_expr_attributes)]

use std::cell::RefCell;

use engage::app::goddata::{GodData, IGodDataMethods};
use engage::app::godpool::GodPool;
use engage::app::godunit::IGodUnitMethods;
use engage::app::procdesc::ProcDesc;
use engage::app::procinst::ProcInst;
use engage::app::structdata_1::IStructData_1Methods;
use unity::prelude::*;

use engage::List_1Ext;

pub mod api;
pub mod audio;
pub mod bundle;
pub mod catalog;
pub mod combatui;
pub mod combatvibration;
pub mod config;
pub mod graphics;
pub mod injection;
pub mod msbt;
pub mod ringvibration;
pub mod save;
pub mod script;
pub mod sequences;
pub mod sortie;
pub mod sprite;
pub mod support;
pub mod updater;
pub mod utils;
pub mod vibrationevents;
pub mod vibrations;

#[unity::hook("App", "Game", "GetPatchName")]
pub fn get_patch_name_hook(method_info: OptionalMethod) -> Il2CppString {
    let game_version = call_original!(method_info).to_string();
    format!("{}\nCobalt {}", game_version, env!("CARGO_PKG_VERSION")).into()
}

#[unity::hook("App", "Database", "Completed")]
pub fn database_completed_publish_event(method_info: OptionalMethod) {
    call_original!(method_info);
    crate::api::events::publish_system_event(api::events::SystemEvent::GamedataLoaded);
}

#[skyline::hook(offset = 0x281ec70)]
pub fn procinst_jump(this: ProcInst, label: *const i32, method_info: OptionalMethod) {
    let label_value = unsafe { *value_to_int(label) };
    crate::api::events::publish_system_event(api::events::SystemEvent::ProcInstJump {
        proc: this,
        label: label_value,
    });

    call_original!(this, label, method_info)
}

#[unity::from_offset("App", "ProcInst", "CreateInternal")]
pub fn proc_inst_create_internal(
    this: ProcInst,
    parent: ProcInst,
    desc: Array<ProcDesc>,
    name: Il2CppString,
    is_bind: bool,
    method_info: OptionalMethod,
) -> ProcInst;

#[skyline::hook(offset = 0x280a6b0)]
pub fn proc_inst_create_bind(
    this: ProcInst,
    parent: ProcInst,
    desc: Array<ProcDesc>,
    name: Il2CppString,
    method_info: OptionalMethod,
) -> ProcInst {
    let proc = unsafe { proc_inst_create_internal(this, parent, desc, name, true, method_info) };

    crate::api::events::publish_system_event(api::events::SystemEvent::ProcInstBind {
        proc: RefCell::new(this),
        parent: RefCell::new(parent),
    });
    proc
}

#[skyline::from_offset(0x429cc8)]
pub fn value_to_int(value: *const i32) -> *const i32;

pub fn get_gid_from_ascii_name(ascii_name: &str) -> Option<Il2CppString> {
    let needle: Il2CppString = ascii_name.into();
    let list = GodData::get_list();
    if list.is_null() {
        return None;
    }
    list.iter()
        .find(|data| data.get_ascii_name() == needle)
        .map(|data| data.get_gid())
}

// allows for easy remapping of engage zones in the ring polish screen by utilizing the goddata's nickname field.
// makes it consistent with engage zone replacement in engage attacks.
#[unity::hook("App", "GodData", "GetEngageZonePrefabPath")]
pub fn goddata_getengagezoneprefabpath(gid: Il2CppString, method_info: OptionalMethod) -> Il2CppString {
    let god = GodPool::try_get(gid, false);
    if god.is_null() {
        return call_original!(gid, method_info);
    }

    let god_nickname = god.get_data().get_nickname().to_string();
    let Some(stripped_identifier) = god_nickname.split('_').nth(2) else {
        return call_original!(gid, method_info);
    };

    let Some(remapped) = get_gid_from_ascii_name(stripped_identifier) else {
        return call_original!(gid, method_info);
    };

    if remapped == gid {
        return call_original!(gid, method_info);
    }

    println!("[EngageZone] {} => {}", gid, remapped);
    call_original!(remapped, method_info)
}
