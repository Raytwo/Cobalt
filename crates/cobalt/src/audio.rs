use std::sync::atomic::AtomicBool;

use unity::{
    prelude::*,
    system::Dictionary
};

use engage::{
    gamedata::unit::Unit,
    gamesound::GameSound,
    combat::{
        Character,
        CharacterSound
    },
};

use mods::manager::Manager;

use camino::Utf8PathBuf;

pub mod gamesound;
pub mod soundmanager;
pub mod soundplay;
pub mod wwise;

pub const COBALT_EVENT_MARKER_PREFIX: &str = "CobaltEvent:";
pub const ORIGINAL_SUFFIX: &str = "_Original";

pub static PLAY_ORIGINAL_V_PICK: AtomicBool = AtomicBool::new(false);
pub static mut PREVIOUS_LAST_PICK_VOICE: u8 = 0;
pub static mut UNSAFE_CHARACTER_PTR: *const Character = std::ptr::null();

fn get_event_or_fallback(event_name: &Il2CppString) -> &Il2CppString {
    match ParsedVoice::parse(event_name) {
        ParsedVoice::ModdedVoiceEvent(mod_voice) => {
            let mod_string = mod_voice.mod_event.to_string();
            let mod_str = mod_string.as_str();
            let fallback_string = mod_voice.fallback_event.to_string();
            let fallback_str = fallback_string.as_str();

            match mod_str.rsplit_once('_') {
                Some((event_body_str, event_main_str)) => {
                    let event_main = Il2CppString::new(format!("{}_{}", event_body_str, event_main_str));

                    if GameSound::is_event_loaded(event_main) {
                        event_main
                    } else {
                        let event_fallback = Il2CppString::new(format!("{}_{}", event_body_str, fallback_str));
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

enum ParsedVoice<'a> {
    ModdedVoiceEvent(ModdedVoiceEvent<'a>),
    DefaultVoiceEvent(DefaultVoiceEvent<'a>),
}

/// Represents a modded voice name with a fallback option.
///
/// For example, in `Voice="SeasideDragon!PlayerF"`, `SeasideDragon` is the modded voice name,
/// and `PlayerF` is the fallback voice name that will be used if the modded voice is not available.
struct ModdedVoiceEvent<'a> {
    /// The name of the modded voice to be used, e.g., `SeasideDragon`.
    mod_event: &'a Il2CppString,
    /// The fallback voice name, e.g., `PlayerF`, used if the modded voice is unavailable.
    /// This is not necessarily a voice name from the vanilla game - it could be another modded voice name (though this has not been tested).
    fallback_event: &'a Il2CppString,
}

/// Represents an original voice name without any fallbacks.
struct DefaultVoiceEvent<'a> {
    event: &'a Il2CppString,
}

impl ParsedVoice<'_> {
    fn parse(event_name: &Il2CppString) -> ParsedVoice {
        let event_string = event_name.to_string();
        let parts: Vec<&str> = event_string.split('!').collect();

        if parts.len() == 2 {
            return ParsedVoice::ModdedVoiceEvent(ModdedVoiceEvent {
                mod_event: Il2CppString::new(parts[0]),
                fallback_event: Il2CppString::new(parts[1]),
            });
        } else {
            return ParsedVoice::DefaultVoiceEvent(DefaultVoiceEvent { event: event_name });
        }
    }
}

fn get_switchname_fallback(switch_name: &Il2CppString) -> &Il2CppString {
    match switch_name.to_string().split('!').nth(1) {
        Some(switch_fallback) => Il2CppString::new(switch_fallback),
        None => switch_name,
    }
}

#[skyline::hook(offset = 0x1F87570)]
pub fn unitinfo_reservecharavoice(
    side: i32,
    person_switch_name: &Il2CppString,
    engage_switch_name: Option<&Il2CppString>,
    event_name: &Il2CppString,
    method_info: OptionalMethod,
) {
    let event_string = event_name.to_string();
    let person_string = person_switch_name.to_string();

    // println!("[UnitInfo] Event name: {}", event_string);
    // println!("[UnitInfo] Person Switch name: {}", person_string);

    match event_string.as_str() {
        "V_Engage_Respond" => {
            let modded_event = Il2CppString::new(format!("{}_{}", event_string, person_string));

            match GameSound::is_event_loaded(modded_event) {
                true => call_original!(side, person_switch_name, engage_switch_name, modded_event, method_info),
                false => call_original!(side, person_switch_name, engage_switch_name, event_name, method_info),
            }
        },

        _ => call_original!(side, person_switch_name, engage_switch_name, event_name, method_info),
    }
}

// AkMarkerCallbackInfo$$get_strLabel	7102f26bd0	System_String_o * AkMarkerCallbackInfo$$get_strLabel(AkMarkerCallbackInfo_o * __this, MethodInfo * method)	172
#[unity::from_offset("", "AkMarkerCallbackInfo", "get_strLabel")]
fn akmarkercallbackinfo_get_strlabel(this: &(), method_info: OptionalMethod) -> &'static Il2CppString;

// Combat.Character$$get_Sound	7102b00c40	Combat_CharacterSound_o * Combat.Character$$get_Sound(Combat_Character_o * __this, MethodInfo * method)	140
#[unity::from_offset("Combat", "Character", "get_Sound")]
fn combat_character_get_sound(this: &Character, method_info: OptionalMethod) -> &'static CharacterSound;

// Combat.CharacterSound$$PlayVoice	71025f0be0	void Combat.CharacterSound$$PlayVoice(Combat_CharacterSound_o * __this, System_String_o * eventName, MethodInfo * method)	556
#[unity::from_offset("Combat", "CharacterSound", "PlayVoice")]
fn combat_charactersound_play_voice(this: &CharacterSound, event_name: &Il2CppString, method_info: OptionalMethod);

#[unity::from_offset("App", "FileCommon", "GetFullPath")]
fn filecommon_getfullpath(path: &Il2CppString, method_info: OptionalMethod) -> &'static mut Il2CppString;

#[unity::class("App", "FileCommon")]
pub struct FileCommon {}

#[repr(C)]
pub struct FileCommonStaticFields<'a> {
    lock_object: &'a (),
    dictionary: &'a Dictionary<&'a Il2CppString, &'a mut FileData>,
    // Meh
}

#[unity::class("App", "FileHandle")]
pub struct FileHandle {
    data: &'static mut FileData,
}

impl FileHandle {
    pub fn unload(&self) {
        let method = self.get_class().get_method_from_name("Unload", 0).unwrap();

        let unload = unsafe { std::mem::transmute::<_, extern "C" fn(&Self, &MethodInfo)>(method.method_ptr) };

        unload(&self, method);
    }
}

#[unity::class("App", "FileData")]
pub struct FileData {
    state: i32,
    path: &'static Il2CppString,
    data: &'static mut Il2CppArray<u8>,
    refer: &'static mut BindHolder,
}

#[unity::class("App", "BindHolder")]
pub struct BindHolder {
    bind: i32,
}

#[unity::from_offset("App", "BindHolder", "Bind")]
fn bindholder_bind(this: &mut BindHolder, method_info: OptionalMethod) -> bool;

#[skyline::hook(offset = 0x328fff0)]
fn filehandle_loadasync(this: &'static mut FileHandle, path: &Il2CppString, method_info: OptionalMethod) {
    // println!("[FileHandle::LoadAsync] Path: {}", path.to_string());

    let mod_path = Utf8PathBuf::from("Data/StreamingAssets/").join(path.to_string());

    // Check if we have that voiceline file in our mods and load it ourselves if we do
    if Manager::get().exists(&mod_path) {
        // Resets FileHandle
        this.unload();

        let full_path = unsafe { filecommon_getfullpath(path, None) };

        let static_fields = FileCommon::class().get_static_fields_mut::<FileCommonStaticFields>();

        // Check if there's already a cached version of the voiceline
        if static_fields.dictionary.try_get_value(full_path, &mut this.data) == false {
            let mut file = Manager::get().get_file(mod_path).unwrap();

            // Initialize the FileData pointer
            this.data = FileData::instantiate().unwrap();
            this.data.path = full_path;
            this.data.data = Il2CppArray::<u8>::from_slice(&mut file).unwrap();
            this.data.state = 2;
            this.data.refer = BindHolder::instantiate().unwrap();

            // Add our fully loaded file to the cache list
            static_fields.dictionary.add(full_path, this.data);
        }

        unsafe {
            bindholder_bind(this.data.refer, None);
        }
    } else {
        call_original!(this, path, method_info)
    }
}

// Combat.CharacterAppearance$$CreateForSound    7102b0f340    Combat_CharacterAppearance_o * Combat.CharacterAppearance$$CreateForSound(App_Unit_o * unit, MethodInfo * method)    280
#[unity::from_offset("Combat", "CharacterAppearance", "CreateForSound")]
pub fn combat_character_appearance_create_for_sound(unit: &Unit, method_info: OptionalMethod) -> &'static CharacterAppearance;

#[unity::class("App", "AssetTable.Sound")]
pub struct AssetTableSound {
    voice_id: &'static Il2CppString,
    footstep_id: &'static Il2CppString,
    material_id: &'static Il2CppString,
}

#[unity::class("Combat", "CharacterAppearance")]
pub struct CharacterAppearance {
    padding: [u8; 0x88],
    sound: AssetTableSound,
}