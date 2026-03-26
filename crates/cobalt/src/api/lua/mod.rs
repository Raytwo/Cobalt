use std::sync::Mutex;

use unity::prelude::*;

use engage::{
    gamedata::{item::ItemData, unit::Unit, Gamedata, JobData, WeaponMask}, map::image::MapImage, noticemanager::NoticeManager, proc::ProcInst, script::{DynValue, EventResultScriptCommand, EventScript, EventScriptCommand, ScriptUtils}, util::get_instance
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
        event.register_action("MapSetSize", map_set_size);
        event.register_action("MapSetCoords", map_set_coords);

        // Process the callbacks to register lua methods
        EVENTSCRIPT_CB.lock().unwrap().iter().for_each(|cb| cb(event));
    }
}

extern "C" fn map_set_coords(args: &Il2CppArray<&DynValue>, _method_info: OptionalMethod) {
    let x = args.try_get_i32(0);
    let y = args.try_get_i32(1);

    let map_image = get_instance::<MapImage>();
    map_image.playarea_x = x;
    map_image.playarea_z = y;
    map_image.playarea_x1 = x;
    map_image.playarea_z1 = y;


}

#[skyline::from_offset(0x01de2ba0)]
fn mapimage_set_size(ring: &MapImage, w: i32, h: i32, method_info: OptionalMethod);

extern "C" fn map_set_size(args: &Il2CppArray<&DynValue>, _method_info: OptionalMethod) {
    let w = args.try_get_i32(0);
    let h = args.try_get_i32(1);

    let map_image = get_instance::<MapImage>();
    unsafe { mapimage_set_size(&map_image, w, h, None); }
}

#[unity::class("App", "RingData")]
pub struct RingData {}

impl Gamedata for RingData {}

#[skyline::from_offset(0x01c5d420)]
fn unit_ring_pool_add(ring: &Il2CppString, owner: Option<&Unit>, count: i32, method_info: OptionalMethod);

extern "C" fn add_bond_ring(args: &Il2CppArray<&DynValue>, _method_info: OptionalMethod) {
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

extern "C" fn cobaltsystem_notice(args: &Il2CppArray<&DynValue>, _method_info: OptionalMethod) {
    if let Some(string) = args.try_get_string(0) {
        NoticeManager::add(string);
    }
}

extern "C" fn cobaltsystem_has_seasonpass(_args: &Il2CppArray<&DynValue>, _method_info: OptionalMethod) -> &'static DynValue {
    let has_dlc = unsafe { dlcmanager_hascontent(0, None) };
    let dynval = DynValue::new_boolean(has_dlc);

    dynval
}

#[unity::from_offset("App", "DLCManager", "HasContent")]
fn dlcmanager_hascontent(content: i32, method_info: OptionalMethod) -> bool;

extern "C" fn cobaltsystem_unitsetjob(args: &Il2CppArray<&DynValue>, _method_info: OptionalMethod) {
    let pid = match args.try_get_string(0) {
        Some(pid) => pid.to_string(),
        None => {
            NoticeManager::add("UnitSetJob: no PID provided");
            return;
        },
    };

    let jid = match args.try_get_string(1) {
        Some(jid) => jid.to_string(),
        None => {
            NoticeManager::add("UnitSetJob: no JID provided");
            return;
        }
    };

    let unit_data = match args.try_get_unit(0) {
        Some(unit) => unit,
        None => {
            NoticeManager::add(format!("UnitSetJob: No unit found for PID '{}'", pid));
            return;
        }
    };

    let job_data = match JobData::get(&jid) {
        Some(job) => job,
        None => {
            NoticeManager::add(format!("UnitSetJob: Job with JID '{}' does not exist", jid));
            return;
        }
    };

    unit_data.class_change(job_data);

    let weapon_mask_flag = args.try_get_i32(2);

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

extern "C" fn cobaltsystem_unitsetlevel(args: &Il2CppArray<&DynValue>, _method_info: OptionalMethod) {
    let unit_data = args.try_get_unit(0).unwrap_or_else(|| {
        let pid = args.try_get_string(0).unwrap();
        panic!("UnitSetLevel: PID provided ({}) is invalid", pid.to_string());
    });

    let level = args.try_get_i32(1);

    // TODO: Add a check on the level beforehand
    unit_data.set_level(level);
}

extern "C" fn cobaltsystem_unit_learnjobskill(args: &Il2CppArray<&DynValue>, _method_info: OptionalMethod) {
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

extern "C" fn cobaltsystem_itemgainsilent(args: &Il2CppArray<&DynValue>, _method_info: OptionalMethod) {
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

extern "C" fn cobaltsystem_menuitemshopshow(_args: &Il2CppArray<&DynValue>, _method_info: OptionalMethod) {
    let proc = unsafe { scriptutil_getsequence(None) };
    
    unsafe { hubitemshopsequence_createbind(proc, None) }

    unsafe { scriptutil_yield(None) }
}

extern "C" fn cobaltsystem_menuweaponshopshow(_args: &Il2CppArray<&DynValue>, _method_info: OptionalMethod) {
    let proc = unsafe { scriptutil_getsequence(None) };
    
    unsafe { hubweaponshopsequence_createbind(proc, None) }

    unsafe { scriptutil_yield(None) }
}

extern "C" fn cobaltsystem_menurefineshopshow(_args: &Il2CppArray<&DynValue>, _method_info: OptionalMethod) {
    let proc = unsafe { scriptutil_getsequence(None) };
    
    unsafe { hubrefineshopsequence_createbind(proc, None) }

    unsafe { scriptutil_yield(None) }
}

extern "C" fn cobaltsystem_itemgift(args: &Il2CppArray<&DynValue>, _method_info: OptionalMethod) {
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