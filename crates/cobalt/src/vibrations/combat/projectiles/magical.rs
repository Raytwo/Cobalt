use crate::vibrations::queue_handlers::run_vibration_event_by_name;
use crate::vibrations::util::{do_vibrate, get_vibration_judgement, ConnectionType, VibrationJudgement};
use engage::combat::{magicsignalprocessor_get_magic, ArrivalType, HitType, MagicSignalProcessor};
use unity::prelude::*;

// Combat.MagicSignalProcessor$$CmdShoot	7101bf4860	void Combat.MagicSignalProcessor$$CmdShoot(Combat_MagicSignalProcessor_o * __this, Combat_MagicSignal_o * ev, MethodInfo * method)	732
#[skyline::hook(offset = 0x1bf4860)]
pub fn cmd_shoot(this: &MagicSignalProcessor, ev: *const u8, method_info: OptionalMethod) {
    do_vibrate(|| {
        handle_cmd_shoot(this);
    });
    call_original!(this, ev, method_info);
}

/// Handle the vibrations that happen when a magic projectile is thrown.
pub fn handle_cmd_shoot(this: &MagicSignalProcessor) {
    unsafe {
        // Check type of magic it is
        let magic = magicsignalprocessor_get_magic(this, None);
        let magic_bullet_settings = magic.magic_bullet_settings;
        let arrival_type = &magic_bullet_settings.arrival_type;

        let character = this.character;

        let judgement = match arrival_type {
            ArrivalType::ConstantTime => {
                // If it's a miss, playing a vibration will allow us to still feel like we threw some magic, but it just didn't connect.
                if character.get_phase().fields.hit_type.intersects(HitType::Miss) {
                    get_vibration_judgement(character)
                } else {
                    VibrationJudgement::NoVibration
                }
                // else, there's no need to play a casting vibration here since we will instantly connect anyways
            },
            ArrivalType::Flying => {
                // Flying spells are the ones that are thrown and a projectile is spawned
                // Whether we miss or not, we should just play the vibration here
                get_vibration_judgement(character)
            },
        };

        match judgement {
            VibrationJudgement::NoVibration => (),
            VibrationJudgement::ShouldVibrate(connection_type) => match connection_type {
                ConnectionType::Normal => {
                    run_vibration_event_by_name("Cobalt_Projectile_Magic_Normal");
                },
                ConnectionType::Critical => {
                    run_vibration_event_by_name("Cobalt_Projectile_Magic_Critical");
                },
            },
        }
    }
}
