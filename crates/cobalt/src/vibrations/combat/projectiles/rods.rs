use engage::combat::{MagicSignal, MagicSignalProcessor};

use unity::prelude::*;

use crate::vibe_log;
use crate::vibrationevents::get_vibration_event;
use crate::vibrations::queue_handlers::run_vibration_event;
use crate::vibrations::queue_handlers::run_vibration_event_by_name;
use crate::vibrations::util::{do_vibrate, get_vibration_judgement, VibrationJudgement};
use phf::{phf_set, Set};

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
#[skyline::hook(offset = 0x1bf4d40)]
pub fn cmd_sound(this: &MagicSignalProcessor, ev: &MagicSignal, method_info: OptionalMethod) {
    do_vibrate(|| {
        handle_cmd_sound(this, ev);
    });
    call_original!(this, ev, method_info);
}

/// Used for vibrations when a rod is casted.
pub fn handle_cmd_sound(this: &MagicSignalProcessor, ev: &MagicSignal) {
     // only feel casting if "we" are doing it.
     // however, we should always feel the hit from a rod whether it's on the enemy or us.

     if let Some(string_parameter) = ev.string_parameter {
         let sound_name = string_parameter.to_string();
         
         if ROD_CASTING.contains(sound_name.as_str()) {
             match get_vibration_judgement(this.character) {
                 VibrationJudgement::NoVibration => (),
                 VibrationJudgement::ShouldVibrate(_) => {
                     let custom_event = get_vibration_event(format!("Cobalt_{}", sound_name).as_str());
                     vibe_log!("Custom rod event: {:?}", custom_event);
                     match custom_event {
                         Some(event) => {
                             run_vibration_event(&event);
                         },
                         None => {
                             run_vibration_event_by_name("Cobalt_Projectile_Rod");
                         },
                     }
                 },
             }
         }
    }
}
