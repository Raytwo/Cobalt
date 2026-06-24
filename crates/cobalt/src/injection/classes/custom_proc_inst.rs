use engage::app::procinst::*;

#[unity::inject(
    namespace = "Cobalt",
    name = "CustomProcInst",
    parent = ProcInst,
)]
pub struct CustomProcInst {
    celine_babies: u64,
}

#[unity::injected_methods]
impl CustomProcInst {
    pub fn ping(self) {
        println!("[CustomProcInst] ping fired, celine_babies={}", self.celine_babies());
    }

    #[override_virtual(name = "OnDispose")]
    pub fn on_dispose(self) {
        let _ = self;
    }
}

pub fn register() {
    crate::injection::register::<CustomProcInst>();
}
