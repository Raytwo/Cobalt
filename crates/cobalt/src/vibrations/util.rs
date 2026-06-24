use engage::combat::character::{Character, ICharacterMethods};
use engage::combat::phase::IPhaseMethods;
use engage::combat::side::Side;

use engage::GameVariableManager;
use crate::config::combatvibration::COMBAT_VIBRATION_KEY;

/// Determine if we should vibrate or not, and how strong of a vibration it should be.
///
/// Then, if it's a critical, we ask for a stronger vibration to be played.
pub fn get_vibration_judgement(character: Character) -> VibrationJudgement {
    use VibrationJudgement::*;

    if is_vibrating_attack(character) {
            // Play a dramatic vibration when we shoot a critical because I assume the character threw the projectile with more force
        if character.get_phase().get_is_critical() {
            ShouldVibrate(ConnectionType::Critical)
        } else {
            ShouldVibrate(ConnectionType::Normal)
        }
    } else {
        NoVibration
    }
}

/// If it's an ally attack, we vibrate.
/// If it's an enemy attack, we don't.
pub fn is_vibrating_attack(character: Character) -> bool {
    if !character.get_phase().get_is_player_side_attack() {
        return false;
    }

    let side = character.get_side();
    Side::is_chain_atk(side) || Side::is_master(side)
}

/// Lazily execute our vibration and calculations related only if the vibration setting is enabled.
pub fn do_vibrate<F: FnOnce()>(lazy_vibration_logic: F) {
    if GameVariableManager::get_bool(COMBAT_VIBRATION_KEY) {
        lazy_vibration_logic();
    }
}

pub enum ConnectionType {
    Normal,
    Critical,
}

pub enum VibrationJudgement {
    NoVibration,
    ShouldVibrate(ConnectionType),
}
