use std::sync::{LazyLock, RwLock};

use engage::app::reliancedata::{IRelianceDataMethods, RelianceData};
use engage::app::relianceexpdata::{IRelianceExpDataMethods, RelianceExpData};
use engage::app::structdata_1::IStructData_1Methods;
use gamedata::gamedata::SupportBook;
use unity::prelude::*;
use engage::{GameVariableManager, List_1Ext};

use crate::config::supportoutfit::SUPPORT_OUTFIT_KEY;

static CACHE: LazyLock<RwLock<SupportBook>> = LazyLock::new(|| {
    println!("Cache supports");
    let mut book = gamedata::gamedata::SupportBook::new();
    println!("Before parsing support book");

    gamedata::merge::<SupportBook>(&mut book, "patches/xml/Support.xml");
    println!("After parsing support book");
    RwLock::new(book)
});

#[unity::hook("App", "RelianceData", "TryGetExp")]
pub fn reliancedata_trygetexp(this: RelianceData, index: i32, method_info: OptionalMethod) -> RelianceExpData {
    let book = CACHE.read().unwrap();
    let this_pid = this.get_pid().to_string();

    let Some((_, entry)) = book.sets.data.iter().find(|(condition, _)| **condition == this_pid) else {
        return call_original!(this, index, method_info);
    };

    let reliance_list = RelianceData::get_list();
    let reliance_data = reliance_list.get(index);

    let Some(support) = entry.iter().find(|e| e.pid == reliance_data.get_pid().to_string()) else {
        return call_original!(this, index, method_info);
    };

    let Some(exp_type) = support.exp_type else {
        return call_original!(this, index, method_info);
    };

    let exp_list = RelianceExpData::get_list();
    let template = exp_list.get(exp_type as i32);

    let exp = RelianceExpData::new();
    exp.set_rexid(template.get_rexid());
    exp.set_exp_c(template.get_exp_c());
    exp.set_exp_b(template.get_exp_b());
    exp.set_exp_a(template.get_exp_a());
    exp
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
