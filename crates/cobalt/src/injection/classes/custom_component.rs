use engage::unity_engine::monobehaviour::*;

#[unity::inject(
    namespace = "Cobalt",
    name = "CustomComponent",
    parent = MonoBehaviour,
)]
pub struct CustomComponent {
    tick: u64,
}

#[unity::injected_methods]
impl CustomComponent {
    #[rename(name = "Awake")]
    pub fn awake(self) {
        println!("[CustomComponent] Awake fired");
    }

    #[rename(name = "Start")]
    pub fn start(self) {
        println!("[CustomComponent] Start fired (initial tick={})", self.tick());
    }

    #[rename(name = "Update")]
    pub fn update(self) {
        let t = self.tick();
        if t % 60 == 0 {
            println!("[CustomComponent] Update tick={t}");
        }
        self.set_tick(t + 1);
    }

    #[rename(name = "OnDestroy")]
    pub fn on_destroy(self) {
        println!("[CustomComponent] OnDestroy fired (final tick={})", self.tick());
    }
}

pub fn register() {
    crate::injection::register::<CustomComponent>();
}
