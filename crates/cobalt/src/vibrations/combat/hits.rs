use crate::vibe_log;
use crate::vibrations::queue_handlers::run_vibration_event;
use crate::vibrations::util::do_vibrate;

use engage::combat::{
    character_get_phase, phase_get_damage_effect_level, phase_get_is_enemy_side_attack, runtimeanimutil_is_evasion, runtimeanimutil_is_parry,
    AnimationEvent, Character, DamageEffectLevel, Detail, HitType, Phase,
};

use crate::vibrationevents::get_vibration_event;
use crate::vibrations::queue_handlers::run_vibration_event_by_name;
use unity::prelude::*;

#[unity::hook("Combat", "DamageSound", "Play")]
pub fn combat_damagesound_play(attacker: &Character, damager: &Character, phase: &Phase, ev: &AnimationEvent, method_info: OptionalMethod) {
    do_vibrate(|| {
        handle_combat_damagesound_play(attacker, phase, method_info);
    });
    call_original!(attacker, damager, phase, ev, method_info);
}

/// The original entry point for the IPS version of this mod.
/// Based on the amount of damage done, play a vibration.
/// Various other situations like dodging or parrying are also handled here.
/// Surprisingly, this is also where healing vibrations are handled.
/// TODO: Don't play healing vibrations when it's the enemy being healed.
pub fn handle_combat_damagesound_play(attacker: &Character, phase: &Phase, method_info: OptionalMethod) {
    unsafe {
        let is_evasion = runtimeanimutil_is_evasion(phase.damage_hash, method_info);

        if is_evasion {
            let phase = character_get_phase(attacker, method_info);
            let is_enemy_side_attack = phase_get_is_enemy_side_attack(phase, method_info);
            if is_enemy_side_attack {
                // play something for evasions, but only if the player side is the one doing the dodging
                run_vibration_event_by_name("Cobalt_Evasion");
            }
            return;
        }

        let is_parry = runtimeanimutil_is_parry(phase.damage_hash, method_info);

        if is_parry {
            // Play a nice vibration that's just a "ding" when parried.
            run_vibration_event_by_name("Cobalt_Parry");
            return;
        }

        let damage_level: DamageEffectLevel = phase_get_damage_effect_level(phase, method_info);

        let damage_vibration_event_name = match damage_level {
            DamageEffectLevel::High => "Cobalt_DamageEffectLevel_High",
            DamageEffectLevel::Medium => "Cobalt_DamageEffectLevel_Medium",
            DamageEffectLevel::Low => "Cobalt_DamageEffectLevel_Low",
        };

        let Some(event) = get_vibration_event(&damage_vibration_event_name) else {
            vibe_log!("{}: Not found.", damage_vibration_event_name);
            return
        };

        let Some(critical_boost) = get_vibration_event("Cobalt_DamageEffectLevel_CriticalBoost") else {
            vibe_log!("Cobalt_DamageEffectLevel_CriticalBoost: Not found.");
            return
        };

        let Some(engage_boost) = get_vibration_event("Cobalt_DamageEffectLevel_EngageBoost") else {
            vibe_log!("Cobalt_DamageEffectLevel_EngageBoost: Not found.");
            return
        };

        let Some(max) = get_vibration_event("Cobalt_DamageEffectLevel_Maximums") else {
            vibe_log!("Cobalt_DamageEffectLevel_Maximums: Not found.");
            return
        };

        let mut sum = event.to_owned();

        if phase.fields.detail.intersects(Detail::EngageAttack) {
            // Add a little oomph no matter what the damage level for engage attacks
            // Values were somewhat randomly chosen and "felt right" at the time.
            sum = sum + engage_boost.clone();
            vibe_log!("{} + {}", damage_vibration_event_name, "Cobalt_DamageEffectLevel_EngageBoost");
        }

        if phase.fields.hit_type.intersects(HitType::Critical) || phase.fields.detail.intersects(Detail::Smash) {
            // Add a little oomph no matter what the damage level
            // Values were somewhat randomly chosen and "felt right" at the time.
            sum = sum + critical_boost.clone();
            vibe_log!("{} + {}", damage_vibration_event_name, "Cobalt_DamageEffectLevel_CriticalBoost");
        }

        sum.time = f32::clamp(sum.time, 0.0, max.time);
        sum.amp_high = f32::clamp(sum.amp_high, 0.0, max.amp_high);
        sum.amp_low = f32::clamp(sum.amp_low, 0.0, max.amp_low);
        sum.amplitude_magnitude = f32::clamp(sum.amplitude_magnitude, 0.0, max.amplitude_magnitude);

        run_vibration_event(&sum);
    }
}
