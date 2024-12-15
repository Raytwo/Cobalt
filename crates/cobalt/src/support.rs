use std::sync::{RwLock, LazyLock};

use unity::prelude::*;
use gamedata::gamedata::SupportBook;
use engage::gamedata::Gamedata;
use engage::gamevariable::GameVariableManager;
use crate::config::supportoutfit::SUPPORT_OUTFIT_KEY;

#[unity::class("App", "RelianceExpData")]
pub struct RelianceExpData {
    base: [u8; 0x10],
    rexid: &'static Il2CppString,
    exp_c: u8,
    exp_b: u8,
    exp_a: u8,
}

#[unity::class("App", "RelianceData")]
pub struct RelianceData {
    base: [u8; 0x10],
    pid: &'static Il2CppString,
    exp_type: &'static Il2CppArray<u8>,
}

impl Gamedata for RelianceData { }

impl Gamedata for RelianceExpData { }

static CACHE: LazyLock<RwLock<SupportBook>> = LazyLock::new(|| {
    println!("Cache supports");
    let mut book = gamedata::gamedata::SupportBook::new();
    println!("Before parsing support book");

    gamedata::merge::<SupportBook>(&mut book, "patches/xml/Support.xml");
    println!("After parsing support book");
    RwLock::new(book)
});

#[unity::hook("App", "RelianceData", "TryGetExp")]
pub fn reliancedata_trygetexp(
    this: &'static RelianceData,
    index: i32,
    method_info: OptionalMethod,
) -> Option<&'static mut RelianceExpData> {
    // println!("PID: {}, Index: {}", this.pid.to_string(), index);
    let book = CACHE.read().unwrap();

    match book.sets.data.iter().find(|(condition, _)| **condition == this.pid.to_string()) {
        Some((_, entry)) => {
            let reliance_data = RelianceData::get_list()
                .unwrap()
                .get(index as usize)
                .unwrap();

            entry
                .into_iter()
                .find(|entry| entry.pid == reliance_data.pid.to_string())
                .map(|entry| {
                    let exp_type = entry.exp_type.unwrap();
                    
                    RelianceExpData::get_list()
                        .unwrap()
                        .get(exp_type as usize)
                        .map(|entry| {
                            let exp = RelianceExpData::instantiate().unwrap();
                            exp.rexid = entry.rexid;
                            exp.exp_c = entry.exp_c;
                            exp.exp_b = entry.exp_b;
                            exp.exp_a = entry.exp_a;
                            exp
                        })
                }).flatten()
        },
        None => call_original!(this, index, method_info)
    }
}

#[skyline::hook(offset = 0x1bb5f60)]
pub fn get_for_demo_hook(pid: &Il2CppString, is_default: bool, is_plain: bool, method_info: OptionalMethod) -> *const u8 {
    let default = if GameVariableManager::get_bool(SUPPORT_OUTFIT_KEY) {
        false
    } else {
        is_default
    };

    call_original!(pid, default, is_plain, method_info)
}