use std::cell::RefCell;
use std::sync::{
    LazyLock, RwLock
};
use engage::proc::ProcInst;

mod event;
pub use event::*;
use semver::Version;
use semver_macro::env_version;

pub type SystemEventHandler = extern "C" fn(&Event<SystemEvent>);
pub type CobaltEventHandler = extern "C" fn(&Event<CobaltEvent>);

pub static SYSTEM_PUBLISHER: LazyLock<RwLock<EventPublisher<SystemEvent>>> = LazyLock::new(|| RwLock::new(EventPublisher::new()));
pub static COBALT_PUBLISHER: LazyLock<RwLock<EventPublisher<CobaltEvent>>> = LazyLock::new(|| RwLock::new(EventPublisher::new()));

const COBALT_VERSION: Version = env_version!("CARGO_PKG_VERSION");

#[repr(C)]
pub enum SystemEvent {
    CatalogLoaded,
    GamedataLoaded,
    MsbtLoaded,
    LanguageChanged,
    SaveLoaded { ty: i32, slot_id: i32 },
    ProcInstJump { proc: &'static ProcInst, label: i32 },
    ProcInstBind {
        proc: RefCell<&'static mut ProcInst>,
        parent: RefCell<Option<&'static mut ProcInst>>,
    },
}

#[repr(C)]
pub enum CobaltEvent {
    Dummy,
}

#[repr(C)]
pub struct CobApiVersion {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
}

impl From<Version> for CobApiVersion {
    fn from(value: Version) -> Self {
        CobApiVersion { major: value.major, minor: value.minor, patch: value.patch }
    }
}

#[no_mangle]
pub extern "C" fn cobapi_cobalt_version() -> CobApiVersion {
    COBALT_VERSION.into()
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