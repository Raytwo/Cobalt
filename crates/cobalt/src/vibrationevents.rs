use astra_formats::AstraBook;
use easer::functions::*;
use gamedata::gamedata::{EasingType, VibrationEvent, VibrationEventBook, VibrationEventChain};
use std::collections::HashMap;
use std::collections::VecDeque;
use std::path::Path;
use std::sync::LazyLock;
use std::sync::RwLock;

/// Perhaps these two hashmaps could be combined into one?
pub static VIBRATION_EVENTS: LazyLock<RwLock<HashMap<String, VibrationEvent>>> = LazyLock::new(|| RwLock::new(HashMap::new()));

/// Perhaps these two hashmaps could be combined into one?
pub static VIBRATION_EVENT_CHAINS: LazyLock<RwLock<HashMap<String, VibrationEventChain>>> = LazyLock::new(|| RwLock::new(HashMap::new()));

pub static VIBRATION_EVENT_QUEUE: LazyLock<RwLock<VecDeque<QueuedVibrationEvent>>> = LazyLock::new(|| RwLock::new(VecDeque::new()));

pub static mut USE_VIBRATION_XML_DEVELOPER_MODE: bool = false;

const VIBRATION_XML_PATH: &str = "sd:/engage/VibrationEvent.xml";

#[derive(Clone)]
pub struct QueuedVibrationEvent {
    pub vibration_event: VibrationEvent,
    pub scheduled_game_time: f32,
}

pub fn initialize_vibration_data() {
    let path = Path::new(VIBRATION_XML_PATH);
    if path.exists() {
        println!("VibrationEvent.xml detected - turning on developer mode for vibration events.");
        unsafe {
            USE_VIBRATION_XML_DEVELOPER_MODE = true;
            load_vibration_event_data();
        }
    } else {
        println!("VibrationEvent.xml not detected - using bundled vibration events only.");
        update_hashmap(&get_built_in_book())
    }
}

fn get_built_in_book() -> VibrationEventBook {
    let built_in_xml = include_str!("../resources/VibrationEvent.xml");
    AstraBook::from_string(built_in_xml).unwrap()
}

/// This is used for hot-reloading vibration events while the game is running.
/// I've tried to make it so nothing crashes if the XML is malformed or missing.
pub fn load_vibration_event_data() {
    unsafe {
        if !USE_VIBRATION_XML_DEVELOPER_MODE {
            return;
        }
    }
    println!("Hot-reloading vibration event data...");
    let parsing = std::time::Instant::now();
    let vibration_event_book: Result<VibrationEventBook, _> = AstraBook::load(VIBRATION_XML_PATH);
    let vibration_event_book = match vibration_event_book {
        Ok(data) => data,
        Err(err) => {
            println!("Failed to load VibrationEvent.xml: {}. Loading from bundled file instead.", err);
            get_built_in_book()
        },
    };

    println!("Parsing vibration data took took {}ms", parsing.elapsed().as_millis());
    update_hashmap(&vibration_event_book);
}

fn update_hashmap(vibration_event_book: &VibrationEventBook) {
    let convert_to_hashmap = std::time::Instant::now();
    if let Ok(mut map) = VIBRATION_EVENTS.try_write() {
        map.clear();
        vibration_event_book.vibration_events.data.iter().for_each(|entry| {
            map.insert(entry.name.to_owned(), (*entry).clone().into());
        });
    }
    if let Ok(mut chain_map) = VIBRATION_EVENT_CHAINS.try_write() {
        chain_map.clear();
        vibration_event_book.vibration_event_chains.data.iter().for_each(|entry| {
            chain_map.insert(entry.name.to_owned(), (*entry).clone().into());
        });
    }
    println!(
        "Putting VibrationEvents and VibrationEventChains into the hashmaps took {}ms",
        convert_to_hashmap.elapsed().as_millis()
    );
}

#[macro_export]
macro_rules! vibe_log {
    ($($x:tt)*) => {
        #[allow(unused_unsafe)]
        unsafe {
            use crate::vibrationevents::USE_VIBRATION_XML_DEVELOPER_MODE;
            if USE_VIBRATION_XML_DEVELOPER_MODE {
                print!("VibrationEvent -- ");
                println!($($x)*);
            }
        }
    }
}

pub struct EasingArgs {
    /// the current time (or position) of the tween. This can be seconds or frames, steps, seconds, ms, whatever as long as the unit is the same as is used for the total time.
    pub current_time: f32,
    /// the beginning value of the property.
    pub beginning_value: f32,
    /// chage is the change between the beginning and destination value of the property.
    pub change: f32,
    /// total time of the tween.
    pub total_time: f32,
}

/// Execute an easing function with the given arguments.
pub fn execute_easing(easing_type: Option<EasingType>, args: EasingArgs) -> f32 {
    use EasingType::*;
    let EasingArgs {
        current_time: t,
        beginning_value: b,
        change: c,
        total_time: d,
    } = args;
    let Some(easing_type) = easing_type else {
        return b;
    };
    match easing_type {
        EaseInExpo => Expo::ease_in(t, b, c, d),
        EaseInQuint => Quint::ease_in(t, b, c, d),
        EaseInQuad => Quad::ease_in(t, b, c, d),
        EaseInCubic => Cubic::ease_in(t, b, c, d),
        EaseOutCubic => Cubic::ease_out(t, b, c, d),
        EaseOutSine => Sine::ease_out(t, b, c, d),
        EaseInBounce => Bounce::ease_in(t, b, c, d),
        // Reverse types start from 0.0 and work their way up to the beginning value.
        ReverseEaseInExpo => Expo::ease_in(t, 0.0, b, d),
        ReverseEaseInQuint => Quint::ease_in(t, 0.0, b, d),
        ReverseEaseInQuad => Quad::ease_in(t, 0.0, b, d),
        ReverseEaseInCubic => Cubic::ease_in(t, 0.0, b, d),
        ReverseEaseOutCubic => Cubic::ease_out(t, 0.0, b, d),
        ReverseEaseOutSine => Sine::ease_out(t, 0.0, b, d),
        ReverseEaseInBounce => Bounce::ease_in(t, 0.0, b, d),
    }
}

pub fn get_event_from<T: Clone>(event_name: &str, rwlock_map: &LazyLock<RwLock<HashMap<String, T>>>) -> Option<T> {
    match rwlock_map.try_read() {
        Ok(map) => {
            let event = map.get(event_name).clone();
            if let Some(event) = event {
                return Some(event.clone());
            } else {
                return None;
            }
        },
        Err(_) => {
            vibe_log!("Failed to read event map");
            return None;
        },
    };
}

pub fn get_vibration_event(event_name: &str) -> Option<VibrationEvent> {
    return get_event_from(event_name, &VIBRATION_EVENTS);
}

pub fn get_vibration_event_chain(event_name: &str) -> Option<VibrationEventChain> {
    return get_event_from(event_name, &VIBRATION_EVENT_CHAINS);
}
