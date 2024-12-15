use engage::combat::{
    character_get_side, phase_get_is_critical, phase_get_is_player_side_attack, side_is_chain_atk, side_is_master, Character,
};

use crate::config::combatvibration::COMBAT_VIBRATION_KEY;
use engage::gamevariable::GameVariableManager;

/// Determine if we should vibrate or not, and how strong of a vibration it should be.
///
/// Then, if it's a critical, we ask for a stronger vibration to be played.
pub fn get_vibration_judgement(character: &Character) -> VibrationJudgement {
    use VibrationJudgement::*;

    if is_vibrating_attack(character) {
        match unsafe { phase_get_is_critical(character.get_phase(), None) } {
            // Play a dramatic vibration when we shoot a critical because I assume the character threw the projectile with more force
            true => ShouldVibrate(ConnectionType::Critical),
            false => ShouldVibrate(ConnectionType::Normal),
        }
    } else {
        NoVibration
    }
}

/// If it's an ally attack, we vibrate.
/// If it's an enemy attack, we don't.
pub fn is_vibrating_attack(character: &Character) -> bool {
    if unsafe { phase_get_is_player_side_attack(character.get_phase(), None) } {
        let side = unsafe { character_get_side(character, None) };

        if unsafe { side_is_chain_atk(side, None) || side_is_master(side, None) } {
            // chain attacks no matter from whom should vibrate - some cases this might be an emblem but whatever
            // if we're here, we're a master unit (not an emblem)
            true
        } else {
            false
        }
    } else {
        // Don't play any vibrations if it's not a player side attack - it feels weird feeling the enemy's shots because they are psychologically not "ours".
        false
    }
}

/// Lazily execute our vibration and calculations related only if the vibration setting is enabled.
pub fn do_vibrate<F>(lazy_vibration_logic: F)
where
    F: Fn(),
{
    let vibration_enabled: bool = GameVariableManager::get_bool(COMBAT_VIBRATION_KEY);
    match vibration_enabled {
        true => lazy_vibration_logic(),
        false => (),
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
