use std::cell::RefCell;
use std::sync::LazyLock;
use std::sync::RwLock;

use cobapi::services::ProcInstBindArgs;
use cobapi::services::ProcInstJumpArgs;
use cobapi::services::SaveLoadedArgs;
use cobapi::services::SystemEventService;
pub use cobapi::CobApiVerison;
pub use cobapi::Event;
pub use cobapi::SystemEvent;

use semver::Version;
use semver_macro::env_version;

mod event;
pub use event::*;

#[repr(C)]
pub enum CobaltEvent {
    Dummy,
}

pub type SystemEventHandler = extern "C" fn(&Event<SystemEvent>);
pub type CobaltEventHandler = extern "C" fn(&Event<CobaltEvent>);

pub static SYSTEM_PUBLISHER: LazyLock<RwLock<EventPublisher<SystemEvent>>> =
    LazyLock::new(|| RwLock::new(EventPublisher::new()));
pub static COBALT_PUBLISHER: LazyLock<RwLock<EventPublisher<CobaltEvent>>> =
    LazyLock::new(|| RwLock::new(EventPublisher::new()));

const COBALT_VERSION: Version = env_version!("CARGO_PKG_VERSION");

// 0.5.0
// Typed-service bridge, heavily inspired by Reloaded II's Controller API but Rust style.
// At the moment, only the SystemPublisher API is exposed as a Service to test how robust the system is.
// It is recommended to NOT use this as it might change before 2.0.0

struct Slot<F: 'static> {
    handlers: RwLock<Vec<F>>,
}

impl<F: 'static> Slot<F> {
    const fn new() -> Self {
        Self { handlers: RwLock::new(Vec::new()) }
    }
}

impl<F: PartialEq + Copy + 'static> Slot<F> {
    fn subscribe(&self, h: F) {
        self.handlers.write().unwrap().push(h);
    }

    fn unsubscribe(&self, h: F) {
        self.handlers.write().unwrap().retain(|x| *x != h);
    }
}

impl<A> Slot<extern "C" fn(&A)> {
    fn fire(&self, args: &A) {
        // Copy them to avoid a deadlock
        let snapshot = self.handlers.read().unwrap().clone();

        for h in snapshot {
            h(args);
        }
    }
}

impl Slot<extern "C" fn()> {
    fn fire(&self) {
        // Copy them to avoid a deadlock
        let snapshot = self.handlers.read().unwrap().clone();

        for h in snapshot {
            h();
        }
    }
}

struct SystemEventBridge {
    save_loaded: Slot<extern "C" fn(&SaveLoadedArgs)>,
    catalog_loaded: Slot<extern "C" fn()>,
    gamedata_loaded: Slot<extern "C" fn()>,
    msbt_loaded: Slot<extern "C" fn()>,
    language_changed: Slot<extern "C" fn()>,
    proc_inst_jump: Slot<extern "C" fn(&ProcInstJumpArgs)>,
    proc_inst_bind: Slot<extern "C" fn(&ProcInstBindArgs)>,
}

impl SystemEventBridge {
    const fn new() -> Self {
        Self {
            save_loaded: Slot::new(),
            catalog_loaded: Slot::new(),
            gamedata_loaded: Slot::new(),
            msbt_loaded: Slot::new(),
            language_changed: Slot::new(),
            proc_inst_jump: Slot::new(),
            proc_inst_bind: Slot::new(),
        }
    }
}

impl SystemEventService for &'static SystemEventBridge {
    fn subscribe_save_loaded(&self, h: extern "C" fn(&SaveLoadedArgs)) { self.save_loaded.subscribe(h); }
    fn unsubscribe_save_loaded(&self, h: extern "C" fn(&SaveLoadedArgs)) { self.save_loaded.unsubscribe(h); }

    fn subscribe_catalog_loaded(&self, h: extern "C" fn()) { self.catalog_loaded.subscribe(h); }
    fn unsubscribe_catalog_loaded(&self, h: extern "C" fn()) { self.catalog_loaded.unsubscribe(h); }

    fn subscribe_gamedata_loaded(&self, h: extern "C" fn()) { self.gamedata_loaded.subscribe(h); }
    fn unsubscribe_gamedata_loaded(&self, h: extern "C" fn()) { self.gamedata_loaded.unsubscribe(h); }

    fn subscribe_msbt_loaded(&self, h: extern "C" fn()) { self.msbt_loaded.subscribe(h); }
    fn unsubscribe_msbt_loaded(&self, h: extern "C" fn()) { self.msbt_loaded.unsubscribe(h); }

    fn subscribe_language_changed(&self, h: extern "C" fn()) { self.language_changed.subscribe(h); }
    fn unsubscribe_language_changed(&self, h: extern "C" fn()) { self.language_changed.unsubscribe(h); }

    fn subscribe_proc_inst_jump(&self, h: extern "C" fn(&ProcInstJumpArgs)) { self.proc_inst_jump.subscribe(h); }
    fn unsubscribe_proc_inst_jump(&self, h: extern "C" fn(&ProcInstJumpArgs)) { self.proc_inst_jump.unsubscribe(h); }

    fn subscribe_proc_inst_bind(&self, h: extern "C" fn(&ProcInstBindArgs)) { self.proc_inst_bind.subscribe(h); }
    fn unsubscribe_proc_inst_bind(&self, h: extern "C" fn(&ProcInstBindArgs)) { self.proc_inst_bind.unsubscribe(h); }
}

static SYSTEM_EVENT_BRIDGE: SystemEventBridge = SystemEventBridge::new();

pub fn install_event_services() {
    let _ = cobapi::services::install(&SYSTEM_EVENT_BRIDGE);
}

#[no_mangle]
pub extern "C" fn cobapi_cobalt_version() -> CobApiVerison {
    CobApiVerison {
        major: COBALT_VERSION.major,
        minor: COBALT_VERSION.minor,
        patch: COBALT_VERSION.patch,
    }
}

pub fn publish_system_event(event: SystemEvent) {
    // Call the callbacks for the service
    match &event {
        SystemEvent::SaveLoaded { ty, slot_id } => {
            SYSTEM_EVENT_BRIDGE.save_loaded.fire(&SaveLoadedArgs { ty: *ty, slot_id: *slot_id });
        }
        SystemEvent::CatalogLoaded => SYSTEM_EVENT_BRIDGE.catalog_loaded.fire(),
        SystemEvent::GamedataLoaded => SYSTEM_EVENT_BRIDGE.gamedata_loaded.fire(),
        SystemEvent::MsbtLoaded => SYSTEM_EVENT_BRIDGE.msbt_loaded.fire(),
        SystemEvent::LanguageChanged => SYSTEM_EVENT_BRIDGE.language_changed.fire(),
        SystemEvent::ProcInstJump { proc, label } => {
            SYSTEM_EVENT_BRIDGE.proc_inst_jump.fire(&ProcInstJumpArgs { proc: *proc, label: *label });
        }
        SystemEvent::ProcInstBind { proc, parent } => {
            SYSTEM_EVENT_BRIDGE.proc_inst_bind.fire(&ProcInstBindArgs {
                proc: RefCell::new(*proc.borrow()),
                parent: RefCell::new(*parent.borrow()),
            });
        }
        _ => {}
    }

    // Call the callbacks for the old API
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