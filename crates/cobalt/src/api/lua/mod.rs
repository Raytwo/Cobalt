use std::sync::Mutex;

use unity::prelude::*;

use engage::{
    gamedata::{item::ItemData, unit::Unit, Gamedata, JobData, WeaponMask}, noticemanager::NoticeManager, proc::ProcInst, script::{DynValue, EventResultScriptCommand, EventScript, EventScriptCommand, ScriptUtils}
};

#[unity::from_offset("App", "ScriptUtil", "GetSequence")]
pub extern "C" fn scriptutil_getsequence(_method_info: OptionalMethod) -> &'static ProcInst;

#[unity::from_offset("App", "ScriptUtil", "Yield")]
pub extern "C" fn scriptutil_yield(_method_info: OptionalMethod);

pub type EventScriptRegistrationCallback = extern "C" fn(&EventScript);

pub static EVENTSCRIPT_CB: Mutex<Vec<EventScriptRegistrationCallback>> = Mutex::new(Vec::new());

#[no_mangle]
pub extern "C" fn cobapi_register_eventscript_cb(callback: EventScriptRegistrationCallback) {
    println!("CobAPI received a EventScript Registration callback");

    let mut pending_calls = EVENTSCRIPT_CB.lock().unwrap();
    pending_calls.push(callback);
}

pub struct ScriptCobalt;

impl ScriptCobalt {
    pub fn register(event: &EventScript) {
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

        // Process the callbacks to register lua methods
        EVENTSCRIPT_CB.lock().unwrap().iter().for_each(|cb| cb(event));
    }
}

#[unity::class("App", "RingData")]
pub struct RingData {}

impl Gamedata for RingData {}

#[skyline::from_offset(0x01c5d420)]
fn unit_ring_pool_add(ring: &Il2CppString, owner: Option<&Unit>, count: i32, method_info: OptionalMethod);

extern "C" fn add_bond_ring(args: &Il2CppArray<DynValue>, _method_info: OptionalMethod) {
    if let Some(rnid) = args.try_get_string(0) {
        RingData::get(&rnid.to_string()).unwrap_or_else(|| {
            panic!("AddBondRing: Ring with RNID'{}' does not exist", rnid.to_string());
        });

        let number = args.try_get_i32(1);

        if number < 1 {
            unsafe { unit_ring_pool_add(rnid, None, 1, None); }
        } else {
            unsafe { unit_ring_pool_add(rnid, None, number, None); }
        }
    }
}

extern "C" fn cobaltsystem_notice(args: &Il2CppArray<DynValue>, _method_info: OptionalMethod) {
    let string = args.try_get_string(0).unwrap();
    NoticeManager::add(string);
}

extern "C" fn cobaltsystem_has_seasonpass(_args: &Il2CppArray<DynValue>, _method_info: OptionalMethod) -> &'static DynValue {
    let has_dlc = unsafe { dlcmanager_hascontent(0, None) };
    let dynval = DynValue::new_boolean(has_dlc);

    dynval
}

#[unity::from_offset("App", "DLCManager", "HasContent")]
fn dlcmanager_hascontent(content: i32, method_info: OptionalMethod) -> bool;

extern "C" fn cobaltsystem_unitsetjob(args: &Il2CppArray<DynValue>, _method_info: OptionalMethod) {
    let unit_data = args.try_get_unit(0).unwrap_or_else(|| {
        let pid = args.try_get_string(0).unwrap();
        panic!("UnitSetJob: PID provided ({}) is invalid", pid.to_string());
    });

    let jid = args.try_get_string(1);
    let weapon_mask_flag = args.try_get_i32(2);

    let job_data = JobData::get(&jid.unwrap().to_string()).unwrap_or_else(|| {
        let jid = args.try_get_string(1).unwrap();
        panic!("UnitSetJob: Job with JID '{}' does not exist", jid.to_string());
    });

    unit_data.class_change(job_data);

    // If argument is empty
    if weapon_mask_flag == i32::MAX {
        return
    }

    if weapon_mask_flag < 1 {
        NoticeManager::add("UnitSetJob: Flags lower than 1 are invalid");
        return
    }

    if weapon_mask_flag > 0x3FF {
        NoticeManager::add("UnitSetJob: Flags higher than 1023/0x3FF are invalid");
        return
    }

    let weapon_mask = WeaponMask::instantiate().unwrap();
    weapon_mask.fields.value = weapon_mask_flag;

    unit_data.set_selected_weapon(weapon_mask);
}

extern "C" fn cobaltsystem_unitsetlevel(args: &Il2CppArray<DynValue>, _method_info: OptionalMethod) {
    let unit_data = args.try_get_unit(0).unwrap_or_else(|| {
        let pid = args.try_get_string(0).unwrap();
        panic!("UnitSetLevel: PID provided ({}) is invalid", pid.to_string());
    });

    let level = args.try_get_i32(1);

    // TODO: Add a check on the level beforehand
    unit_data.set_level(level);
}

extern "C" fn cobaltsystem_unit_learnjobskill(args: &Il2CppArray<DynValue>, _method_info: OptionalMethod) {
    let unit_data = args.try_get_unit(0).unwrap_or_else(|| {
        let pid = args.try_get_string(0).unwrap();
        panic!("UnitSetJob: PID provided ({}) is invalid", pid.to_string());
    });

    let job_data = unit_data.get_job();

    // TODO: Check if the job is valid

    unit_data.learn_job_skill(job_data);
}

#[unity::class("App", "UnitItem")]
pub struct UnitItem { }

#[unity::from_offset("App", "Transporter", "Add")]
pub fn app_transporter_add(unititem: &UnitItem, method_info: OptionalMethod);

#[unity::from_offset("App", "UnitItem", ".ctor")]
pub fn unititem_ctor(this: &UnitItem, item: &ItemData, method_info: OptionalMethod);

extern "C" fn cobaltsystem_itemgainsilent(args: &Il2CppArray<DynValue>, _method_info: OptionalMethod) {
    let unit_data = args.try_get_unit(0);
    let item_data = args.try_get_item(1).unwrap_or_else(|| {
        let iid = args.try_get_string(1).unwrap();
        panic!("ItemGainSilent: IID provided ({}) is invalid", iid.to_string());
    });
    
    match unit_data {
        Some(unit_data) => unit_data.add_item(item_data),
        None => {
            let unititem = UnitItem::instantiate().unwrap();
            unsafe { unititem_ctor(&unititem, item_data, None) }
            unsafe { app_transporter_add(&unititem, None) }
            unsafe { scriptutil_yield(None) }
        },
    }
}

extern "C" fn cobaltsystem_menuitemshopshow(_args: &Il2CppArray<DynValue>, _method_info: OptionalMethod) {
    let proc = unsafe { scriptutil_getsequence(None) };
    
    unsafe { hubitemshopsequence_createbind(proc, None) }

    unsafe { scriptutil_yield(None) }
}

extern "C" fn cobaltsystem_menuweaponshopshow(_args: &Il2CppArray<DynValue>, _method_info: OptionalMethod) {
    let proc = unsafe { scriptutil_getsequence(None) };
    
    unsafe { hubweaponshopsequence_createbind(proc, None) }

    unsafe { scriptutil_yield(None) }
}

extern "C" fn cobaltsystem_menurefineshopshow(_args: &Il2CppArray<DynValue>, _method_info: OptionalMethod) {
    let proc = unsafe { scriptutil_getsequence(None) };
    
    unsafe { hubrefineshopsequence_createbind(proc, None) }

    unsafe { scriptutil_yield(None) }
}

extern "C" fn cobaltsystem_itemgift(args: &Il2CppArray<DynValue>, _method_info: OptionalMethod) {
    let proc = unsafe { scriptutil_getsequence(None) };

    let reward_id = args.try_get_string(0).unwrap();
    let message_id = args.try_get_string(1).unwrap();

    unsafe { hubsequence_giftget(proc, reward_id, message_id, None) };

    unsafe { scriptutil_yield(None) }
}

#[unity::from_offset("App", "HubSequence", "GiftGet")]
pub fn hubsequence_giftget(this: &ProcInst, reward_id: &'static Il2CppString, message_id: &'static Il2CppString, _method_info: OptionalMethod);

#[unity::from_offset("App", "HubItemShopSequence", "CreateBind")]
pub fn hubitemshopsequence_createbind(parent: &ProcInst, _method_info: OptionalMethod);

#[unity::from_offset("App", "HubWeaponShopSequence", "CreateBind")]
pub fn hubweaponshopsequence_createbind(parent: &ProcInst, _method_info: OptionalMethod);

#[unity::from_offset("App", "HubRefineShopSequence", "CreateBind")]
pub fn hubrefineshopsequence_createbind(parent: &ProcInst, _method_info: OptionalMethod);