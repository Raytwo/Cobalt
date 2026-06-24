use std::sync::atomic::AtomicBool;

use camino::Utf8PathBuf;
use engage::combat::character::Character;
use engage::combat::charactersound::CharacterSound;
use engage::app::bindholder::{BindHolder, IBindHolderMethods};
use engage::app::filecommon::{FileCommon, FileCommon_State};
use engage::app::filedata::{FileData, IFileData};
use engage::app::gamesound::GameSound;
use engage::system::collections::generic::dictionary_2::IDictionary_2Methods as IDictionaryMethods;
use unity::{prelude::*, IlInstance};

use engage::{FileHandle, IFileHandle};

pub mod gamesound;
pub mod soundmanager;
pub mod soundplay;
pub mod wwise;

pub const COBALT_EVENT_MARKER_PREFIX: &str = "CobaltEvent:";
pub const ORIGINAL_SUFFIX: &str = "_Original";

pub static PLAY_ORIGINAL_V_PICK: AtomicBool = AtomicBool::new(false);
pub static mut PREVIOUS_LAST_PICK_VOICE: i8 = 0;
pub static mut UNSAFE_CHARACTER_PTR: *mut () = std::ptr::null_mut();

fn get_event_or_fallback(event_name: Il2CppString) -> Il2CppString {
    match ParsedVoice::parse(event_name) {
        ParsedVoice::ModdedVoiceEvent(mod_voice) => {
            let mod_string = mod_voice.mod_event.to_string();
            let mod_str = mod_string.as_str();
            let fallback_string = mod_voice.fallback_event.to_string();
            let fallback_str = fallback_string.as_str();

            match mod_str.rsplit_once('_') {
                Some((event_body_str, event_main_str)) => {
                    let event_main: Il2CppString = format!("{}_{}", event_body_str, event_main_str).as_str().into();

                    if GameSound::is_event_loaded(event_main) {
                        event_main
                    } else {
                        let event_fallback: Il2CppString = format!("{}_{}", event_body_str, fallback_str).as_str().into();
                        event_fallback
                    }
                },
                None => event_name,
            }
        },
        ParsedVoice::DefaultVoiceEvent(default_voice) => {
            println!("Getting Default voice: {}", default_voice.event);
            default_voice.event
        },
    }
}

pub(crate) enum ParsedVoice {
    ModdedVoiceEvent(ModdedVoiceEvent),
    DefaultVoiceEvent(DefaultVoiceEvent),
}

/// Represents a modded voice name with a fallback option.
///
/// For example, in `Voice="SeasideDragon!PlayerF"`, `SeasideDragon` is the modded voice name,
/// and `PlayerF` is the fallback voice name that will be used if the modded voice is not available.
pub(crate) struct ModdedVoiceEvent {
    pub mod_event: Il2CppString,
    pub fallback_event: Il2CppString,
}

/// Represents an original voice name without any fallbacks.
pub(crate) struct DefaultVoiceEvent {
    pub event: Il2CppString,
}

impl ParsedVoice {
    pub fn parse(event_name: Il2CppString) -> ParsedVoice {
        let event_string = event_name.to_string();
        let parts: Vec<&str> = event_string.split('!').collect();

        if parts.len() == 2 {
            ParsedVoice::ModdedVoiceEvent(ModdedVoiceEvent {
                mod_event: parts[0].into(),
                fallback_event: parts[1].into(),
            })
        } else {
            ParsedVoice::DefaultVoiceEvent(DefaultVoiceEvent { event: event_name })
        }
    }
}

pub(crate) fn get_switchname_fallback(switch_name: Il2CppString) -> Il2CppString {
    match switch_name.to_string().split('!').nth(1) {
        Some(switch_fallback) => switch_fallback.into(),
        None => switch_name,
    }
}

#[skyline::hook(offset = 0x1F87570)]
pub fn unitinfo_reservecharavoice(
    side: i32,
    person_switch_name: Il2CppString,
    engage_switch_name: Il2CppString,
    event_name: Il2CppString,
    method_info: OptionalMethod,
) {
    let event_string = event_name.to_string();
    let person_string = person_switch_name.to_string();

    // println!("[UnitInfo] Event name: {}", event_string);
    // println!("[UnitInfo] Person Switch name: {}", person_string);

    match event_string.as_str() {
        "V_Engage_Respond" => {
            let modded_event: Il2CppString = format!("{}_{}", event_string, person_string).as_str().into();

            match GameSound::is_event_loaded(modded_event) {
                true => call_original!(side, person_switch_name, engage_switch_name, modded_event, method_info),
                false => call_original!(side, person_switch_name, engage_switch_name, event_name, method_info),
            }
        },

        _ => call_original!(side, person_switch_name, engage_switch_name, event_name, method_info),
    }
}

// AkMarkerCallbackInfo$$get_strLabel	7102f26bd0	System_String_o * AkMarkerCallbackInfo$$get_strLabel(AkMarkerCallbackInfo_o * __this, MethodInfo * method)	172
// Combat.Character$$get_Sound	7102b00c40	Combat_CharacterSound_o * Combat.Character$$get_Sound(Combat_Character_o * __this, MethodInfo * method)	140
// Combat.CharacterSound$$PlayVoice	71025f0be0	void Combat.CharacterSound$$PlayVoice(Combat_CharacterSound_o * __this, System_String_o * eventName, MethodInfo * method)	556
#[unity::from_offset("", "AkMarkerCallbackInfo", "get_strLabel")]
pub(crate) fn akmarkercallbackinfo_get_strlabel(this: IlInstance, method_info: OptionalMethod) -> Il2CppString;

#[unity::from_offset("Combat", "Character", "get_Sound")]
pub(crate) fn combat_character_get_sound(this: Character, method_info: OptionalMethod) -> CharacterSound;

#[unity::from_offset("Combat", "CharacterSound", "PlayVoice")]
pub(crate) fn combat_charactersound_play_voice(this: CharacterSound, event_name: Il2CppString, method_info: OptionalMethod);

#[skyline::hook(offset = 0x328fff0)]
fn filehandle_loadasync(this: FileHandle, path: Il2CppString, method_info: OptionalMethod) {
    let mod_path = Utf8PathBuf::from("Data/StreamingAssets/").join(path.to_string());

    // Check if we have that voiceline file in our mods and load it ourselves if we do
    if !mods::contains(&mod_path) {
        return call_original!(this, path, method_info);
    }

    // Resets FileHandle
    this.unload();

    let full_path = FileCommon::get_full_path(path);
    let dictionary = FileCommon::s_dictionary();

    if dictionary.contains_key(full_path) {
        this.set_m_data(dictionary.get_item(full_path));
    } else {
        let mut file = mods::read(mod_path).unwrap();

        // Initialize the FileData pointer
        let data = FileData::new();
        data.set_m_path(full_path);
        data.set_m_data(Array::<u8>::from_slice(&mut file).unwrap());
        data.set_m_state(FileCommon_State::done());
        data.set_m_refer(BindHolder::new());
        this.set_m_data(data);

        dictionary.add(full_path, data);
    }

    this.m_data().m_refer().bind();
}
