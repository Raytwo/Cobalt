use std::sync::{
    LazyLock, RwLock
};

use engage::proc::ProcInst;

mod event;
pub use event::*;

pub type SystemEventHandler = extern "C" fn(&Event<SystemEvent>);
pub type CobaltEventHandler = extern "C" fn(&Event<CobaltEvent>);

pub static SYSTEM_PUBLISHER: LazyLock<RwLock<EventPublisher<SystemEvent>>> = LazyLock::new(|| RwLock::new(EventPublisher::new()));
pub static COBALT_PUBLISHER: LazyLock<RwLock<EventPublisher<CobaltEvent>>> = LazyLock::new(|| RwLock::new(EventPublisher::new()));

#[repr(C)]
pub enum SystemEvent {
    CatalogLoaded,
    GamedataLoaded,
    MsbtLoaded,
    LanguageChanged,
    SaveLoaded { ty: i32, slot_id: i32 },
    ProcInstJump { proc: &'static ProcInst, label: i32 }
}

#[repr(C)]
pub enum CobaltEvent {
    Dummy,
}

pub fn publish_system_event(event: SystemEvent) {
    SYSTEM_PUBLISHER.read().unwrap().publish_event(&Event::Args(event));
}

pub fn publish_cobalt_event(event: CobaltEvent) {
    COBALT_PUBLISHER.read().unwrap().publish_event(&Event::Args(event));
}

#[no_mangle]
pub extern "C" fn cobapi_register_system_event_listener(callback: SystemEventHandler) {
    println!("CobAPI received a System event listener");

    let mut publisher = SYSTEM_PUBLISHER.write().unwrap();
    publisher.subscribe_handler(callback);
}

#[no_mangle]
pub extern "C" fn cobapi_unregister_system_event_listener(callback: SystemEventHandler) {
    println!("CobAPI received a System event listener");

    let mut publisher = SYSTEM_PUBLISHER.write().unwrap();
    publisher.unsubscribe_handler(callback);
}

#[no_mangle]
pub extern "C" fn cobapi_register_cobaltevent_listener(callback: CobaltEventHandler) {
    println!("CobAPI received a Cobalt event listener");

    let mut publisher = COBALT_PUBLISHER.write().unwrap();
    publisher.subscribe_handler(callback);
}

#[no_mangle]
pub extern "C" fn cobapi_unregister_cobalt_event_listener(callback: CobaltEventHandler) {
    println!("CobAPI received a Cobalt event listener");

    let mut publisher = COBALT_PUBLISHER.write().unwrap();
    publisher.unsubscribe_handler(callback);
}