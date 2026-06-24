use engage::combat::character::ICharacterMethods;
use engage::combat::magic::IMagic;
use engage::combat::magicarrivaltype::MagicArrivalType;
use engage::combat::magicbulletsettings::IMagicBulletSettings;
use engage::combat::magicsignal::MagicSignal;
use engage::combat::magicsignalprocessor::{IMagicSignalProcessor, IMagicSignalProcessorMethods, MagicSignalProcessor};
use engage::combat::phase::{IPhase, Phase_HitType};
use engage::prelude::Cast;
use unity::{OptionalMethod};

use crate::vibrations::queue_handlers::run_vibration_event_by_name;
use crate::vibrations::util::{do_vibrate, get_vibration_judgement, ConnectionType, VibrationJudgement};

// Combat.MagicSignalProcessor$$CmdShoot	7101bf4860	void Combat.MagicSignalProcessor$$CmdShoot(Combat_MagicSignalProcessor_o * __this, Combat_MagicSignal_o * ev, MethodInfo * method)	732
#[unity::hook("Combat", "MagicSignalProcessor", "CmdShoot")]
pub fn cmd_shoot(this: MagicSignalProcessor, ev: MagicSignal, method_info: OptionalMethod) {
    do_vibrate(|| handle_cmd_shoot(this));
    call_original!(this, ev, method_info);
}

/// Handle the vibrations that happen when a magic projectile is thrown.
pub fn handle_cmd_shoot(this: MagicSignalProcessor) {
    let magic = this.get_magic();
    if magic.is_null() {
        return;
    }
    let settings = magic.bullet_settings();
    if settings.is_null() {
        return;
    }
    let character = this.cp();

    let arrival = settings.arrival_type();
    let judgement = if arrival == MagicArrivalType::constant_time() {
        if character.get_phase().hit_type().value & Phase_HitType::miss().value != 0 {
            get_vibration_judgement(character)
        } else {
            VibrationJudgement::NoVibration
        }
    } else if arrival == MagicArrivalType::flying() {
        get_vibration_judgement(character)
    } else {
        VibrationJudgement::NoVibration
    };

    match judgement {
        VibrationJudgement::NoVibration => (),
        VibrationJudgement::ShouldVibrate(connection) => match connection {
            ConnectionType::Normal => run_vibration_event_by_name("Cobalt_Projectile_Magic_Normal"),
            ConnectionType::Critical => run_vibration_event_by_name("Cobalt_Projectile_Magic_Critical"),
        },
    }
}
