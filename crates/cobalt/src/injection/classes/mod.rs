pub mod custom_component;
pub mod custom_proc_inst;

pub fn register_all() {
    custom_proc_inst::register();
    custom_component::register();
}
