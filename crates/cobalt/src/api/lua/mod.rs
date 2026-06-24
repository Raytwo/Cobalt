use std::sync::Mutex;

use engage::app::dlcmanager::{DLCManager, DLCManager_Content};
use engage::app::eventscript::EventScript;
use engage::app::hubitemshopsequence::HubItemShopSequence;
use engage::app::hubrefineshopsequence::HubRefineShopSequence;
use engage::app::hubsequence::{HubSequence, IHubSequenceMethods};
use engage::app::hubweaponshopsequence::HubWeaponShopSequence;
use engage::app::itemdata::ItemData;
use engage::app::jobdata::JobData;
use engage::app::mapimage::{IMapImageMethods, MapImage};
use engage::app::noticemanager::{NoticeManager, NoticeManager_Kinds};
use engage::app::ringdata::RingData;
use engage::app::scriptutil::ScriptUtil;
use engage::app::singletonclass_1::ISingletonClass_1Methods;
use engage::app::structdata_1::IStructData_1Methods;
use engage::app::transporter::Transporter;
use engage::app::unit::{IUnitMethods, Unit};
use engage::app::unititem::{IUnitItemMethods, UnitItem};
use engage::app::unitringpool::UnitRingPool;
use engage::app::weaponmask::WeaponMask;
use engage::moon_sharp::interpreter::dynvalue::DynValue;
use unity::prelude::*;
use unity::FromIlInstance;

use engage::{DynValueArgs, EventScriptExt};

pub type EventScriptRegistrationCallback = extern "C" fn(EventScript);

pub static EVENTSCRIPT_CB: Mutex<Vec<EventScriptRegistrationCallback>> = Mutex::new(Vec::new());

#[no_mangle]
pub extern "C" fn cobapi_register_eventscript_cb(callback: EventScriptRegistrationCallback) {
    println!("CobAPI received a EventScript Registration callback");

    let mut pending_calls = EVENTSCRIPT_CB.lock().unwrap();
    pending_calls.push(callback);
}

pub struct ScriptCobalt;

impl ScriptCobalt {
    pub fn register(event: EventScript) {
        // Notice(string)
        event.register_action("CobaltNotice", cobaltsystem_notice);
        // ItemGainSilent(pid, iid)
        event.register_action("ItemGainSilent", cobaltsystem_itemgainsilent);
        // UnitSetJob(pid, jid)
        event.register_action("UnitSetJob", cobaltsystem_unitsetjob);
        // UnitSetLevel(pid, number)
        event.register_action("UnitSetLevel", cobaltsystem_unitsetlevel);
        // UnitLearnJobSkill(pid)
        event.register_action("UnitLearnJobSkill", cobaltsystem_unit_learnjobskill);
        // MenuItemShopShow()
        event.register_action("MenuItemShopShow", cobaltsystem_menuitemshopshow);
        // MenuWeaponShopShow()
        event.register_action("MenuWeaponShopShow", cobaltsystem_menuweaponshopshow);
        // MenuRefineShopShow()
        event.register_action("MenuRefineShopShow", cobaltsystem_menurefineshopshow);
        // ItemGift(reward_id, message_id)
        event.register_action("ItemGift", cobaltsystem_itemgift);
        // local x = HasPurchasedSeasonPass() -- bool
        event.register_function("HasPurchasedSeasonPass", cobaltsystem_has_seasonpass);
        event.register_action("AddBondRing", add_bond_ring);
        event.register_action("MapSetSize", map_set_size);
        event.register_action("MapSetCoords", map_set_coords);

        // Snapshot the callback list under the lock, then drop the lock before
        // invoking. Prevents self-deadlock if a plugin callback re-enters
        // cobapi_register_eventscript_cb (or any other path that grabs the same
        // Mutex) on the same thread.
        let callbacks: Vec<EventScriptRegistrationCallback> = EVENTSCRIPT_CB.lock().unwrap().clone();
        for (_i, cb) in callbacks.iter().enumerate() {
            cb(event);
        }
    }
}

extern "C" fn map_set_coords(args: Array<DynValue>, _method_info: OptionalMethod) {
    let x = args.try_get_i32(0);
    let y = args.try_get_i32(1);

    let map_image = MapImage::get_instance();
    map_image.set_play_area_x(x);
    map_image.set_play_area_z(y);
    map_image.set_play_area_x1(x);
    map_image.set_play_area_z1(y);
}

extern "C" fn map_set_size(args: Array<DynValue>, _method_info: OptionalMethod) {
    let w = args.try_get_i32(0);
    let h = args.try_get_i32(1);

    MapImage::get_instance().set_size(w, h);
}

extern "C" fn add_bond_ring(args: Array<DynValue>, _method_info: OptionalMethod) {
    let rnid = args.try_get_string(0);
    if rnid.is_null() {
        return;
    }

    let ring = RingData::get(rnid);
    if ring.is_null() {
        panic!("AddBondRing: Ring with RNID '{}' does not exist", rnid);
    }

    let number = args.try_get_i32(1);
    let count = if number < 1 { 1 } else { number };

    UnitRingPool::add(rnid, Unit::from_il_instance(IlInstance::null()), count);
}

extern "C" fn cobaltsystem_notice(args: Array<DynValue>, _method_info: OptionalMethod) {
    let string = args.try_get_string(0);
    
    if !string.is_null() {
        NoticeManager::add(NoticeManager_Kinds::facility(), string);
    }
}

extern "C" fn cobaltsystem_has_seasonpass(
    _args: Array<DynValue>,
    _method_info: OptionalMethod,
) -> DynValue {
    let has_dlc = DLCManager::has_content(DLCManager_Content::e0());
    DynValue::new_boolean(has_dlc)
}

extern "C" fn cobaltsystem_unitsetjob(args: Array<DynValue>, _method_info: OptionalMethod) {
    let pid = args.try_get_string(0);
    if pid.is_null() {
        NoticeManager::add(NoticeManager_Kinds::none(), Il2CppString::from("UnitSetJob: no PID provided"));
        return;
    }

    let jid = args.try_get_string(1);
    if jid.is_null() {
        NoticeManager::add(NoticeManager_Kinds::none(), Il2CppString::from("UnitSetJob: no JID provided"));
        return;
    }

    let unit_data = args.try_get_unit(0);
    if unit_data.is_null() {
        NoticeManager::add(
            NoticeManager_Kinds::none(),
            Il2CppString::from(format!("UnitSetJob: No unit found for PID '{}'", pid)),
        );
        return;
    }

    let job_data = JobData::get(jid);
    if job_data.is_null() {
        NoticeManager::add(
            NoticeManager_Kinds::none(),
            Il2CppString::from(format!("UnitSetJob: Job with JID '{}' does not exist", jid)),
        );
        return;
    }

    unit_data.class_change(job_data, ItemData::from_il_instance(IlInstance::null()));

    let weapon_mask_flag = args.try_get_i32(2);

    // If argument is empty
    if weapon_mask_flag == i32::MAX {
        return;
    }

    if weapon_mask_flag < 1 {
        NoticeManager::add(
            NoticeManager_Kinds::none(),
            Il2CppString::from("UnitSetJob: Flags lower than 1 are invalid"),
        );
        return;
    }

    if weapon_mask_flag > 0x3FF {
        NoticeManager::add(
            NoticeManager_Kinds::none(),
            Il2CppString::from("UnitSetJob: Flags higher than 1023/0x3FF are invalid"),
        );
        return;
    }

    let weapon_mask = WeaponMask::new_2(weapon_mask_flag);
    unit_data.set_selected_weapon(weapon_mask);
}

extern "C" fn cobaltsystem_unitsetlevel(args: Array<DynValue>, _method_info: OptionalMethod) {
    let unit_data = args.try_get_unit(0);
    if unit_data.is_null() {
        let pid = args.try_get_string(0);
        panic!("UnitSetLevel: PID provided ({}) is invalid", pid);
    }

    let level = args.try_get_i32(1);
    // TODO: Add a check on the level beforehand
    unit_data.set_level(level);
}

extern "C" fn cobaltsystem_unit_learnjobskill(args: Array<DynValue>, _method_info: OptionalMethod) {
    let unit_data = args.try_get_unit(0);
    if unit_data.is_null() {
        let pid = args.try_get_string(0);
        panic!("UnitSetJob: PID provided ({}) is invalid", pid);
    }

    let _job_data = unit_data.get_job();
    // TODO: Check if the job is valid
    unit_data.learn_job_skill();
}

extern "C" fn cobaltsystem_itemgainsilent(args: Array<DynValue>, _method_info: OptionalMethod) {
    let unit_data = args.try_get_unit(0);
    let item_data = args.try_get_item(1);
    if item_data.is_null() {
        let iid = args.try_get_string(1);
        panic!("ItemGainSilent: IID provided ({}) is invalid", iid);
    }

    if !unit_data.is_null() {
        unit_data.item_add_3(item_data);
    } else {
        let unit_item = <UnitItem as FromIlInstance>::instantiate()
            .expect("UnitItem::instantiate");
        UnitItem::ctor_2(unit_item, item_data);
        Transporter::add(unit_item);
        ScriptUtil::r#yield();
    }
}

extern "C" fn cobaltsystem_menuitemshopshow(_args: Array<DynValue>, _method_info: OptionalMethod) {
    let proc = ScriptUtil::get_sequence();
    HubItemShopSequence::create_bind(proc);
    ScriptUtil::r#yield();
}

extern "C" fn cobaltsystem_menuweaponshopshow(
    _args: Array<DynValue>,
    _method_info: OptionalMethod,
) {
    let proc = ScriptUtil::get_sequence();
    HubWeaponShopSequence::create_bind(proc);
    ScriptUtil::r#yield();
}

extern "C" fn cobaltsystem_menurefineshopshow(
    _args: Array<DynValue>,
    _method_info: OptionalMethod,
) {
    let proc = ScriptUtil::get_sequence();
    HubRefineShopSequence::create_bind(proc);
    ScriptUtil::r#yield();
}

extern "C" fn cobaltsystem_itemgift(args: Array<DynValue>, _method_info: OptionalMethod) {
    let proc = ScriptUtil::get_sequence();
    let reward_id = args.try_get_string(0);
    let message_id = args.try_get_string(1);
    let hub_seq = <HubSequence as FromIlInstance>::from_il_instance(proc.as_instance());
    hub_seq.gift_get(reward_id, message_id);
    ScriptUtil::r#yield();
}
