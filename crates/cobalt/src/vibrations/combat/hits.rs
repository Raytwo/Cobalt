use engage::combat::character::{Character, ICharacterMethods};
use engage::combat::phase::{IPhase, IPhaseMethods, Phase, Phase_Detail, Phase_HitType};
use engage::combat::runtimeanimutil::RuntimeAnimUtil;
use engage::unity_engine::animationevent::AnimationEvent;
use unity::OptionalMethod;

use crate::vibe_log;
use crate::vibrationevents::get_vibration_event;
use crate::vibrations::queue_handlers::{run_vibration_event, run_vibration_event_by_name};
use crate::vibrations::util::do_vibrate;

#[unity::hook("Combat", "DamageSound", "Play")]
pub fn combat_damagesound_play(
    attacker: Character,
    damager: Character,
    phase: Phase,
    ev: AnimationEvent,
    method_info: OptionalMethod,
) {
    do_vibrate(|| handle_combat_damagesound_play(attacker, phase));
    call_original!(attacker, damager, phase, ev, method_info);
}

/// The original entry point for the IPS version of this mod.
/// Based on the amount of damage done, play a vibration.
/// Various other situations like dodging or parrying are also handled here.
/// Surprisingly, this is also where healing vibrations are handled.
/// TODO: Don't play healing vibrations when it's the enemy being healed.
pub fn handle_combat_damagesound_play(attacker: Character, phase: Phase) {
    let damage_hash = phase.damage_hash();

    if RuntimeAnimUtil::is_evasion(damage_hash) {
        if attacker.get_phase().get_is_enemy_side_attack() {
            run_vibration_event_by_name("Cobalt_Evasion");
        }
        return;
    }

    if RuntimeAnimUtil::is_parry(damage_hash) {
        run_vibration_event_by_name("Cobalt_Parry");
        return;
    }

    let damage_level = phase.get_damage_effect_level();
    let damage_event_name = match damage_level {
        2 => "Cobalt_DamageEffectLevel_High",
        1 => "Cobalt_DamageEffectLevel_Medium",
        0 => "Cobalt_DamageEffectLevel_Low",
        _ => "Cobalt_DamageEffectLevel_Low",
    };

    let Some(base_event) = get_vibration_event(damage_event_name) else {
        vibe_log!("{}: Not found.", damage_event_name);
        return;
    };
    let Some(critical_boost) = get_vibration_event("Cobalt_DamageEffectLevel_CriticalBoost") else {
        vibe_log!("Cobalt_DamageEffectLevel_CriticalBoost: Not found.");
        return;
    };
    let Some(engage_boost) = get_vibration_event("Cobalt_DamageEffectLevel_EngageBoost") else {
        vibe_log!("Cobalt_DamageEffectLevel_EngageBoost: Not found.");
        return;
    };
    let Some(max) = get_vibration_event("Cobalt_DamageEffectLevel_Maximums") else {
        vibe_log!("Cobalt_DamageEffectLevel_Maximums: Not found.");
        return;
    };

    let mut sum = base_event.to_owned();
    let detail = phase.detail();
    let hit_type = phase.hit_type();

    if detail.value & Phase_Detail::engage_attack().value != 0 {
        sum = sum + engage_boost.clone();
        vibe_log!("{} + {}", damage_event_name, "Cobalt_DamageEffectLevel_EngageBoost");
    }

    if hit_type.value & Phase_HitType::critical().value != 0
        || detail.value & Phase_Detail::smash().value != 0
    {
        sum = sum + critical_boost.clone();
        vibe_log!("{} + {}", damage_event_name, "Cobalt_DamageEffectLevel_CriticalBoost");
    }

    sum.time = f32::clamp(sum.time, 0.0, max.time);
    sum.amp_high = f32::clamp(sum.amp_high, 0.0, max.amp_high);
    sum.amp_low = f32::clamp(sum.amp_low, 0.0, max.amp_low);
    sum.amplitude_magnitude = f32::clamp(sum.amplitude_magnitude, 0.0, max.amplitude_magnitude);

    run_vibration_event(&sum);
}
