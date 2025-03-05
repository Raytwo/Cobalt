#![feature(ptr_sub_ptr)]
#![feature(stmt_expr_attributes)]
#![feature(unsafe_cell_from_mut)]

use std::{
    cell::RefCell,
    collections::HashMap,
    ffi::CStr,
    sync::{LazyLock, RwLock},
};

use engage::{
    gamedata::{Gamedata, GodData},
    godpool::god_pool_try_get_gid,
    proc::{ProcInst, ProcInstFields},
};
use il2cpp::assembly::Il2CppImage;
use unity::prelude::*;

pub mod api;
pub mod audio;
pub mod bundle;
pub mod catalog;
pub mod combatui;
pub mod combatvibration;
pub mod config;
pub mod graphics;
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
pub fn get_patch_name_hook(method_info: OptionalMethod) -> &'static mut Il2CppString {
    let game_version = call_original!(method_info).to_string();

    format!("{}\nCobalt {}", game_version, env!("CARGO_PKG_VERSION"),).into()
}

#[unity::class("App", "MapSequence")]
pub struct MapSequence {
    proc: ProcInstFields,
    is_resume: bool,
    is_loaded: bool,
    scene_name: Option<&'static Il2CppString>,
    scene_mode: i32,
    is_completed: bool,
}

#[skyline::hook(offset = 0x281ec70)]
pub fn procinst_jump(this: &'static ProcInst, label: &i32, method_info: OptionalMethod) {
    crate::api::events::publish_system_event(api::events::SystemEvent::ProcInstJump {
        proc: this,
        label: unsafe { *value_to_int(label) },
    });

    call_original!(this, label, method_info)
}

#[skyline::from_offset(0x429cc8)]
pub fn value_to_int(value: &i32) -> &i32;

pub fn get_gid_from_ascii_name(ascii_name: &str) -> Option<&Il2CppString> {
    GodData::get_list()?.iter()
        .find(|data| data.get_ascii_name() == Some(ascii_name.into()))
        .map(|data| data.gid)
}

// allows for easy remapping of engage zones in the ring polish screen by utilizing the goddata's nickname field.
// makes it consistent with engage zone replacement in engage attacks.
#[skyline::hook(offset = 0x232E960)]
pub fn goddata_getengagezoneprefabpath(gid: &Il2CppString, method_info: OptionalMethod) -> &'static Il2CppString {
    let god_data = unsafe { god_pool_try_get_gid(gid, false, method_info) };
    match god_data {
        Some(god) => {
            let god_nickname = god.data.nickname.to_string();
            let stripped_identifier = match god_nickname.split('_').nth(2) {
                Some(identifier) => identifier,
                None => return call_original!(gid, method_info),
            };

            match get_gid_from_ascii_name(stripped_identifier) {
                Some(remapped_gid) => {
                    if gid == remapped_gid {
                        return call_original!(gid, method_info);
                    }

                    println!("[EngageZone] {} => {}", gid, remapped_gid);
                    call_original!(remapped_gid, method_info)
                }
                _ => call_original!(gid, method_info),
            }
        },
        None => call_original!(gid, method_info),
    }
}