use crate::vibe_log;
use engage::{gametime::get_time, vibrationmanager::vibrate};
use gamedata::gamedata::VibrationEvent;
use unity::prelude::*;

use crate::vibrationevents::{
    execute_easing, get_vibration_event, get_vibration_event_chain, EasingArgs, QueuedVibrationEvent, VIBRATION_EVENT_QUEUE,
};
use crate::vibrations::hooks::IS_REWINDING;
use crate::vibrations::util::do_vibrate;

//App.FieldBgmManager$$Tick	7102d56090	void App.FieldBgmManager$$Tick(App_FieldBgmManager_o * __this, MethodInfo * method)	168
/// Process the vibration queue on every tick. There is probably a better place for this, but this was the most convenient.
#[skyline::hook(offset = 0x2d56090)]
pub fn field_bgm_manager_tick(this: *const u8, method_info: OptionalMethod) -> *const u8 {
    do_vibrate(|| process_queue());
    call_original!(this, method_info)
}

/// Go through the currently queued vibrations and find the last event that is ready to be run.
/// If there were multiple events between the last time this was run and now, only the last one will be run, the others will all be skipped.
pub fn process_queue() {
    if let Ok(mut queue) = VIBRATION_EVENT_QUEUE.try_write() {
        let mut popped_event_count = 0;

        let now = unsafe { get_time() };

        let mut to_run: Option<QueuedVibrationEvent> = None;

        loop {
            match queue.front() {
                Some(peek) => match peek.scheduled_game_time {
                    scheduled_game_time if scheduled_game_time <= now => {
                        to_run = queue.pop_front();
                        popped_event_count += 1;
                    },
                    _ => {
                        vibe_log!(
                            "{}: Not ready to run yet - requested time is {}, now is {}. Break.",
                            peek.vibration_event.name,
                            peek.scheduled_game_time,
                            now
                        );
                        break;
                    },
                },
                None => break,
            }
        }
        
        if let Some(queued_vibration_event) = to_run {
            vibe_log!("Popped {} events.", popped_event_count);
            vibe_log!("{}: Running queued event", queued_vibration_event.vibration_event.name);
            run_vibration_event(&queued_vibration_event.vibration_event);
        };
    }
}

/// Run a vibration event by name, as defined in our XML.
pub fn run_vibration_event_by_name(event_name: &str) {
    if unsafe { IS_REWINDING } {
        // don't play vibrations while rewinding
        vibe_log!("{}: Silencing vibration event due to rewinding.", event_name);
        return;
    }

    if let Some(ref event) = get_vibration_event(event_name) {
        // Simple events that run immediately
        vibe_log!("{}: Running vibration event.", event_name);
        run_vibration_event(&event);
    } else if let Some(ref event_chain) = get_vibration_event_chain(event_name) {
        // More complicated events that need to be queued up.
        unsafe {
            let now = get_time();
            for chain_item in event_chain.chain.iter() {
                let Some(vibration_event) = get_vibration_event(chain_item.name.as_str()) else {
                    vibe_log!("{}: Not found. ", chain_item.name);
                    continue;
                };
                if chain_item.delay == 0.0 {
                    // Immediate events as part of a chain.
                    vibe_log!("{}: Running vibration event.", chain_item.name);
                    run_vibration_event(&vibration_event);
                } else if vibration_event.easing_type.is_some() {
                    // Events that are part of a chain but have easing.
                    vibe_log!("{}: Queuing up vibration event with easing", chain_item.name);
                    queue_eased_vibration(&vibration_event, chain_item.delay);
                } else {
                    // Events that are part of a chain but don't have easing and simply a delay.
                    if let Ok(mut queue) = VIBRATION_EVENT_QUEUE.try_write() {
                        vibe_log!("{}: Queuing up vibration event.", chain_item.name);
                        queue.push_back(QueuedVibrationEvent {
                            vibration_event,
                            scheduled_game_time: now + chain_item.delay,
                        });
                    }
                }
            }
        }
    } else {
        match event_name {
            "Footstep" | "WingFlap" => {
                // blacklisted, these just clog up the logs
            },
            _ => vibe_log!("{}: Not found. ", event_name),
        };
    }
}

pub fn run_vibration_event(vibration_event: &VibrationEvent) {
    if vibration_event.easing_type.is_some() {
        queue_eased_vibration(vibration_event, 0.0);
    } else {
        vibrate(
            vibration_event.time,
            vibration_event.amplitude_magnitude,
            vibration_event.amp_low,
            vibration_event.amp_high,
            vibration_event.freq_low,
            vibration_event.freq_high,
        );
    }
}

const STEPS_PER_SECOND: u32 = 30;
const DURATION_PER_STEP: f32 = 1.0 / STEPS_PER_SECOND as f32;

/// Greedily calculates all the steps needed to ease the vibration event and queues them up.
/// Perhaps an alternate implementation could store a currently being executed vibration(s) and then
/// lazily calculate the next vibraiton to run with easing and summing accounted for when it's needed (on each tick)
pub fn queue_eased_vibration(vibration_event: &VibrationEvent, delay: f32) {
    let vibration_length = vibration_event.time;
    let num_steps = vibration_length * STEPS_PER_SECOND as f32;
    unsafe {
        let now = get_time();
        if let Ok(mut queue) = VIBRATION_EVENT_QUEUE.try_write() {
            vibe_log!("{:?}: Queuing up vibration event to ease", vibration_event);
            for i in 0..num_steps as usize {
                let slice_time = i as f32 * DURATION_PER_STEP;
                let mut vibration_event = vibration_event.clone();
                vibration_event.time = 10.0; // Overprovision time to ensure there aren't any gaps due to lag and the next event not firing yet.
                if i == num_steps as usize - 1 {
                    // last step
                    vibration_event.time = DURATION_PER_STEP; // The last step will be set short to ensure it ends at the right time
                }

                let amp_high_args: EasingArgs = EasingArgs {
                    current_time: slice_time,
                    beginning_value: vibration_event.amp_high,
                    change: -vibration_event.amp_high, // go to zero
                    total_time: vibration_length,
                };

                let amp_low_args: EasingArgs = EasingArgs {
                    current_time: slice_time,
                    beginning_value: vibration_event.amp_low,
                    change: -vibration_event.amp_low, // go to zero
                    total_time: vibration_length,
                };

                vibration_event.amp_high = execute_easing(vibration_event.easing_type, amp_high_args);
                vibration_event.amp_low = execute_easing(vibration_event.easing_type, amp_low_args);
                vibration_event.easing_type = None; // Clear the easing type so we don't try to ease again
                let scheduled_game_time = slice_time + now + delay;
                vibe_log!("{:?}: Queuing up vibration event to run at {}", vibration_event, scheduled_game_time);
                queue.push_back(QueuedVibrationEvent {
                    scheduled_game_time,
                    vibration_event,
                })
            }
        }
    }
}

//Combat.CombatSkip$$Skip	710292ea70	void Combat.CombatSkip$$Skip(Combat_CombatSkip_o * __this, MethodInfo * method)	24
#[skyline::hook(offset = 0x292ea70)]
pub fn combat_skip_skip(this: *const u8, method_info: OptionalMethod) {
    do_vibrate(|| {
        combat_skip_clear_queue();
    });
    call_original!(this, method_info);
}

pub fn combat_skip_clear_queue() {
    vibe_log!("Combat skip, clearing vibration queue.");
    if let Ok(mut queue) = VIBRATION_EVENT_QUEUE.try_write() {
        queue.clear();
    }
}
