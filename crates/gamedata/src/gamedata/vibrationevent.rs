use astra_derive::{Astra, AstraBook};
use astra_formats::Sheet;
use std::fmt;
use std::str::FromStr;
use strum_macros::EnumString;

use ordered_float::OrderedFloat;

use std::cmp::Eq;
use std::ops::Add;

#[derive(AstraBook, Clone)]
pub struct VibrationEventBook {
    pub vibration_events: Sheet<Vec<VibrationEventEntry>>,
    pub vibration_event_chains: Sheet<Vec<VibrationEventChainEntry>>,
}

#[derive(Astra, Debug, Eq, PartialEq, Hash, Clone)]
pub struct VibrationEventChainEntry {
    #[astra(key = "@Name")]
    pub name: String,

    #[astra(key = "@Chain")]
    pub chain: Vec<String>,
}

#[derive(Astra, Debug, Eq, PartialEq, Hash, Clone)]
pub struct VibrationEventEntry {
    #[astra(key = "@Name")]
    pub name: String,
    #[astra(key = "@Time")]
    pub time: Option<OrderedFloat<f32>>,
    #[astra(key = "@AmplitudeMagnitude")]
    pub amplitude_magnitude: Option<OrderedFloat<f32>>,
    #[astra(key = "@AmpLow")]
    pub amp_low: Option<OrderedFloat<f32>>,
    #[astra(key = "@AmpHigh")]
    pub amp_high: Option<OrderedFloat<f32>>,
    #[astra(key = "@FreqLow")]
    pub freq_low: Option<OrderedFloat<f32>>,
    #[astra(key = "@FreqHigh")]
    pub freq_high: Option<OrderedFloat<f32>>,
    #[astra(key = "@Easing")]
    pub easing: Option<String>,
    #[astra(key = "@VibrationType")]
    pub vibration_type: Option<String>,
}

#[derive(Debug, Clone, Copy, EnumString, PartialEq)]
/// Easing types as defined in https://easings.net/
/// "Reverse" types start from 0 and reach their targeted value at the end of the event.
pub enum EasingType {
    EaseInExpo,
    EaseInQuint,
    EaseInQuad,
    EaseInCubic,
    EaseOutCubic,
    EaseOutSine,
    EaseInBounce,
    ReverseEaseInExpo,
    ReverseEaseInQuint,
    ReverseEaseInQuad,
    ReverseEaseInCubic,
    ReverseEaseOutCubic,
    ReverseEaseOutSine,
    ReverseEaseInBounce,
}

#[derive(Debug, Clone, Copy, EnumString, PartialEq)]
pub enum VibrationType {
    Combat,
    MapCombat,
    MapSkillEffect,
    UI,
    EngageAttack,
    Unknown,
}

#[derive(Debug, Clone)]
/// Vibration event covers all the data needed to perform a vibration.
pub struct VibrationEvent {
    pub name: String,
    /// Duration in seconds.
    pub time: f32,
    pub amplitude_magnitude: f32,
    pub amp_low: f32,
    pub amp_high: f32,
    pub freq_low: f32,
    pub freq_high: f32,
    pub easing_type: Option<EasingType>,
    pub vibration_type: Option<VibrationType>,
}

#[derive(Debug, Clone)]
pub struct VibrationEventChainItem {
    pub name: String,
    /// Delay in seconds.
    pub delay: f32,
}

#[derive(Debug, Clone)]
pub struct VibrationEventChain {
    pub name: String,
    pub chain: Vec<VibrationEventChainItem>,
}

// Defaults that the game came with for whatever reason.
pub const FREQ_LOW: f32 = 160.0;
pub const FREQ_HIGH: f32 = 300.0;

impl From<VibrationEventEntry> for VibrationEvent {
    fn from(item: VibrationEventEntry) -> Self {
        VibrationEvent {
            name: item.name,
            time: item.time.unwrap_or(OrderedFloat(0.0)).into_inner(),
            amplitude_magnitude: item.amplitude_magnitude.unwrap_or(OrderedFloat(0.0)).into_inner(),
            amp_low: item.amp_low.unwrap_or(OrderedFloat(0.0)).into_inner(),
            amp_high: item.amp_high.unwrap_or(OrderedFloat(0.0)).into_inner(),
            freq_low: item.freq_low.unwrap_or(OrderedFloat(FREQ_LOW)).into_inner(),
            freq_high: item.freq_high.unwrap_or(OrderedFloat(FREQ_HIGH)).into_inner(),
            easing_type: item.easing.and_then(|value| match parse_enum::<EasingType>(value) {
                Ok(x) => Some(x),
                Err(e) => {
                    println!("Failed to parse easing type: {}", e);
                    None
                },
            }),
            vibration_type: item.vibration_type.and_then(|value| match parse_enum::<VibrationType>(value) {
                Ok(x) => Some(x),
                Err(e) => {
                    println!("Failed to parse vibration type: {}", e);
                    None
                },
            }),
        }
    }
}

enum ParseEnumError {
    UnknownEnumType(String),
}

impl fmt::Display for ParseEnumError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseEnumError::UnknownEnumType(value) => write!(f, "Unknown enum type: {}", value),
        }
    }
}

fn parse_enum<T: FromStr>(value: String) -> Result<T, ParseEnumError> {
    match T::from_str(value.as_str()) {
        Ok(x) => Ok(x),
        Err(_) => Err(ParseEnumError::UnknownEnumType(value)),
    }
}

impl From<VibrationEventChainEntry> for VibrationEventChain {
    fn from(item: VibrationEventChainEntry) -> Self {
        VibrationEventChain {
            name: item.name.to_owned(),
            chain: item
                .chain
                .iter()
                .map(|x| parse_event_chain(x))
                .filter_map(|x| match x {
                    Ok(x) => Some(x),
                    Err(e) => {
                        println!("Failed to parse event chain: {}", e);
                        None
                    },
                })
                .collect(),
        }
    }
}

enum ParseEventChainError {
    InvalidChainString(String),
    InvalidChainStringMissingComma(String),
    InvalidChainStringParseFloat(String),
}

impl fmt::Display for ParseEventChainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseEventChainError::InvalidChainString(value) => write!(f, "Invalid chain string: {}", value),
            ParseEventChainError::InvalidChainStringMissingComma(value) => write!(f, "Invalid chain string, did you forget a comma?: {}", value),
            ParseEventChainError::InvalidChainStringParseFloat(value) => write!(f, "Could not parse the chain string duration to f32: {}", value),
        }
    }
}

fn parse_event_chain(chain_string: &String) -> Result<VibrationEventChainItem, ParseEventChainError> {
    use ParseEventChainError::*;
    let mut split = chain_string.split(",");
    let Some(name) = split.next() else {
        return Err(InvalidChainString(chain_string.to_owned()))
    };
    let Some(delay) = split.next() else {
        return Err(InvalidChainStringMissingComma(chain_string.to_owned()))
    };
    let Some(delay) = delay.parse::<f32>().ok() else {
        return Err(InvalidChainStringParseFloat(chain_string.to_owned()))
    };

    Ok(VibrationEventChainItem {
        name: name.to_owned(),
        delay,
    })
}

impl Add for VibrationEvent {
    type Output = VibrationEvent;

    fn add(self, rhs: VibrationEvent) -> Self::Output {
        Self {
            name: self.name,
            time: self.time + rhs.time,
            amplitude_magnitude: self.amplitude_magnitude + rhs.amplitude_magnitude,
            amp_low: self.amp_low + rhs.amp_low,
            amp_high: self.amp_high + rhs.amp_high,
            freq_low: self.freq_low + rhs.freq_low,
            freq_high: self.freq_high + rhs.freq_high,
            easing_type: self.easing_type,
            vibration_type: self.vibration_type,
        }
    }
}
