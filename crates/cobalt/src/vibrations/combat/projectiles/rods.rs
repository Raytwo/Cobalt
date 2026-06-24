use engage::combat::magicsignal::{IMagicSignal, MagicSignal};
use engage::combat::magicsignalprocessor::{IMagicSignalProcessor, MagicSignalProcessor};
use phf::{phf_set, Set};
use unity::OptionalMethod;

use crate::vibe_log;
use crate::vibrationevents::get_vibration_event;
use crate::vibrations::queue_handlers::{run_vibration_event, run_vibration_event_by_name};
use crate::vibrations::util::{do_vibrate, get_vibration_judgement, VibrationJudgement};

static ROD_CASTING: Set<&'static str> = phf_set! {
    "SE_Magic_Block_01",
    "SE_Magic_Recover_1",
    "SE_Magic_Interfere_1",
    "SE_Magic_Heal_1",
    "SE_Magic_HiHeal_1",
    "SE_Magic_Support_1",
    "SE_Magic_Rest_01",
    "SE_Magic_Torch_1",
    "Play_Magic_WholeHeal_1",
    "Play_Magic_Nosferatu_1",
    "Play_Magic_Nosferatu_4",
};

// Combat.MagicSignalProcessor$$CmdSound	7101bf4d40	void Combat.MagicSignalProcessor$$CmdSound(Combat_MagicSignalProcessor_o * __this, Combat_MagicSignal_o * ev, MethodInfo * method)	372
#[unity::hook("Combat", "MagicSignalProcessor", "CmdSound")]
pub fn cmd_sound(this: MagicSignalProcessor, ev: MagicSignal, method_info: OptionalMethod) {
    do_vibrate(|| handle_cmd_sound(this, ev));
    call_original!(this, ev, method_info);
}

/// Used for vibrations when a rod is casted.
pub fn handle_cmd_sound(this: MagicSignalProcessor, ev: MagicSignal) {
    let string_parameter = ev.string_parameter();
    if string_parameter.is_null() {
        return;
    }

    let sound_name = string_parameter.to_rust_string();
    if !ROD_CASTING.contains(sound_name.as_str()) {
        return;
    }

    if let VibrationJudgement::ShouldVibrate(_) = get_vibration_judgement(this.cp()) {
        let custom_event = get_vibration_event(format!("Cobalt_{sound_name}").as_str());
        vibe_log!("Custom rod event: {:?}", custom_event);
        match custom_event {
            Some(event) => run_vibration_event(&event),
            None => run_vibration_event_by_name("Cobalt_Projectile_Rod"),
        }
    }
}
