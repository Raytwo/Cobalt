#![feature(ptr_sub_ptr)]
#![feature(stmt_expr_attributes)]
#![feature(unsafe_cell_from_mut)]

use std::{
    cell::RefCell,
    collections::HashMap,
    ffi::{CStr, CString},
    os::raw::c_char,
    str::FromStr,
    sync::{LazyLock, RwLock},
};

use camino::Utf8PathBuf;
use engage::{
    combat::{Character, CharacterSound},
    gamedata::{unit::Unit, Gamedata, GodData},
    godpool::god_pool_try_get_gid,
    mapmind::MapMind,
    proc::{ProcInst, ProcInstFields},
    sequence::mainsequence::{MainSequence, MainSequenceLabel, MainSequenceStaticFields},
};
use il2cpp::assembly::Il2CppImage;
use mods::manager::Manager;
use unity::{
    engine::Vector3,
    prelude::*,
    system::{Dictionary, List},
};

pub mod api;
pub mod bundle;
pub mod catalog;
pub mod combatui;
pub mod combatvibration;
pub mod config;
pub mod graphics;
pub mod msbt;
pub mod ringvibration;
pub mod save;
pub mod script;
pub mod sequences;
pub mod sortie;
pub mod sprite;
pub mod support;
pub mod updater;
pub mod utils;
pub mod vibrationevents;
pub mod vibrations;

#[unity::hook("App", "Game", "GetPatchName")]
pub fn get_patch_name_hook(method_info: OptionalMethod) -> &'static mut Il2CppString {
    let game_version = call_original!(method_info).to_string();

    format!("{}\nCobalt {}", game_version, env!("CARGO_PKG_VERSION"),).into()
}

#[unity::class("App", "MapSequence")]
pub struct MapSequence {
    proc: ProcInstFields,
    is_resume: bool,
    is_loaded: bool,
    scene_name: Option<&'static Il2CppString>,
    scene_mode: i32,
    is_completed: bool,
}

#[skyline::hook(offset = 0x281ec70)]
pub fn procinst_jump(this: &'static ProcInst, label: &i32, method_info: OptionalMethod) {
    crate::api::events::publish_system_event(api::events::SystemEvent::ProcInstJump {
        proc: this,
        label: unsafe { *value_to_int(label) },
    });

    call_original!(this, label, method_info)
}

#[skyline::from_offset(0x429cc8)]
pub fn value_to_int(value: &i32) -> &i32;

static mut CLASS_LOOKUP: LazyLock<RwLock<HashMap<(&str, &str), RefCell<&'static mut Il2CppClass>>>> = LazyLock::new(|| RwLock::new(HashMap::new()));

pub fn add_class_to_lookup<Ty: Il2CppClassData>(class: &'static mut Il2CppClass) {
    println!("Registering custom class '{}.{}'", Ty::NAMESPACE, Ty::CLASS);

    unsafe {
        CLASS_LOOKUP.write().unwrap().insert((Ty::NAMESPACE, Ty::CLASS), RefCell::new(class));
    }
}

#[skyline::hook(offset = 0x431220)]
pub fn class_from_name(image: &Il2CppImage, namespace: *const i8, name: *const i8) -> Option<&'static mut Il2CppClass> {
    // println!("[Class::FromName] Namespace '{}', name '{}'", nmspace, nme);
    let res = call_original!(image, namespace, name);
    // println!("[Class::FromName] result: {}", res.is_some());

    if let None = res {
        // Insert injection logic here
        let lock = unsafe { CLASS_LOOKUP.read().unwrap() };

        let nmspace = unsafe { CStr::from_ptr(namespace).to_str().unwrap_or("Unknown") };
        let nme = unsafe { CStr::from_ptr(name).to_str().unwrap_or("Unknown") };

        if let Some(class) = lock.get(&(nmspace, nme)) {
            let ptr = class.as_ptr();
            unsafe { Some(*ptr) }
        } else {
            None
        }
    } else {
        res
    }
}

#[repr(C)]
#[unity::class("App", "GameSound")]
pub struct GameSound {}

pub struct GameSoundStatic {
    default_bank_name_array: &'static mut Il2CppArray<&'static mut Il2CppString>,
}

#[skyline::hook(offset = 0x2287c60)]
pub fn load_default_sound_banks(method_info: OptionalMethod) {
    let static_fields = GameSound::class().get_static_fields_mut::<GameSoundStatic>();
    let mut banks = static_fields.default_bank_name_array.to_vec();

    let manager = mods::manager::Manager::get();

    if let Ok(dir) = manager.get_directory("Data/StreamingAssets/Audio/GeneratedSoundBanks/Switch") {
        if let Ok(filepaths) = manager.get_files_in_directory_and_subdir(dir) {
            for path in filepaths.iter().filter(|path| path.extension() == Some("bnk")) {
                let filename = Il2CppString::new_static(path.file_stem().unwrap());

                if !banks.contains(&filename) {
                    println!("Added '{}' to the list", path);
                    banks.insert(0, filename);
                }
            }
        }
    }

    static_fields.default_bank_name_array = Il2CppArray::from_slice(banks.as_mut_slice()).unwrap();

    // static_fields.default_bank_name_array.iter().for_each(|bank| {
    //     println!("Bank name: {}", bank.to_string());
    // });

    call_original!(method_info);
}

#[repr(C)]
pub struct WwiseFileHolder {
    filesize: i64,
    unk1: u32,
    unk2: u32,
    unk3: u64,
    handle: skyline::nn::fs::FileHandle,
}

#[skyline::hook(offset = 0x169d700)]
pub fn wwise_file_open_hook(path: *const i8, unk1: u64, unk2: *const u8, unk3: &mut WwiseFileHolder) -> u32 {
    let str_path = unsafe { CStr::from_ptr(path).to_str().unwrap() };
    let utfpath = Utf8PathBuf::from_str(str_path).unwrap();

    unsafe {
        let relative_path = utfpath.strip_prefix("rom:/").unwrap();

        let manager = mods::manager::Manager::get();

        let res = if let Some(loc) = manager.get_locations().find(|loc| loc == relative_path) {
            let full_path = manager.get_absolute_full_path(loc).unwrap();
            let new_path = CString::new(full_path.as_str()).unwrap();

            skyline::nn::fs::OpenFile(&mut unk3.handle, new_path.as_c_str().to_bytes_with_nul().as_ptr(), 1)
        } else {
            skyline::nn::fs::OpenFile(&mut unk3.handle, path as *const u8, 1)
        };

        if res == 0 {
            skyline::nn::fs::GetFileSize(&mut unk3.filesize, unk3.handle);
            unk3.unk2 = 0;
            unk3.unk3 = unk1;
            1
        } else {
            2
        }
    }
}

#[skyline::hook(offset = 0x24f9010)]
pub fn wwise_set_state(this: *const u8, valuename: &Il2CppString, method_info: OptionalMethod) {
    // println!("StateGroup name: {}", valuename.to_string());
    call_original!(this, valuename, method_info)
}

#[unity::from_offset("App", "GameSound", "IsEventLoaded")]
fn gamesound_iseventloaded(event_name: &Il2CppString, method_info: OptionalMethod) -> bool;

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

                    if unsafe { gamesound_iseventloaded(event_main, None) } {
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

        _ => event_name
    }
}

fn get_switchname_fallback(switch_name: &Il2CppString) -> &Il2CppString {
    match switch_name.to_string().split('!').nth(1) {
        Some(switch_fallback) => Il2CppString::new(switch_fallback),
        None => switch_name,
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
    
    // fn get_event_or_fallback(event_name: &Il2CppString) -> &Il2CppString {
    //     match ParsedVoice::parse(event_name) {
    //         ParsedVoice::ModdedVoiceEvent(mod_voice) => {
    //             let mod_string = mod_voice.mod_event.to_string();
    //             let mod_str = mod_string.as_str();
    //             let fallback_string = mod_voice.fallback_event.to_string();
    //             let fallback_str = fallback_string.as_str();

    //             match mod_str.rsplit_once('_') {
    //                 Some((event_body_str, event_main_str)) => {
    //                     let event_main = Il2CppString::new(format!("{}_{}", event_body_str, event_main_str));

    //                     if unsafe { gamesound_iseventloaded(event_main, None) } {
    //                         event_main
    //                     } else {
    //                         let event_fallback = Il2CppString::new(format!("{}_{}", event_body_str, fallback_str));
    //                         event_fallback
    //                     }
    //                 },
    //                 None => event_name,
    //             }
    //         },

    //         ParsedVoice::DefaultVoiceEvent(default_voice) => {
    //             println!("Getting Default voice: {}", default_voice.event);
    //             default_voice.event
    //         },

    //         _ => event_name
    //     }
    // }  

    // fn get_voice_name(switch_name: &Il2CppString) -> &Il2CppString {
    //     match ParsedVoice::parse(switch_name) {
    //         ParsedVoice::ModdedVoiceEvent(mod_voice) => {
    //             println!("Getting Fallback voice: {} => {}", switch_name, mod_voice.fallback_event);
    //             mod_voice.fallback_event
    //         },
    //         ParsedVoice::DefaultVoiceEvent(default_voice) => {
    //             println!("Getting Default voice: {}", default_voice.event);
    //             default_voice.event
    //         },
    //     }
    // }
}

pub static mut UNSAFE_CHARACTER_PTR: *const Character = std::ptr::null();

pub const ORIGINAL_SUFFIX: &str = "_Original";

#[skyline::hook(offset = 0x2292270)]
pub fn gamesound_personvoice(
    gameobject: &(),
    person_switch_name: &Il2CppString,
    engage_switch_name: Option<&Il2CppString>,
    event_name: &Il2CppString,
    character: &Character,
    method_info: OptionalMethod,
) {
    let event_string = event_name.to_string();
    // println!(
    //     "[PersonVoice]: Event name: {}, switch name: {}",
    //     event_string,
    //     person_switch_name
    // );

    let character_ptr: *const Character = character as *const Character;
    unsafe {
        UNSAFE_CHARACTER_PTR = character_ptr;
    }

    if event_string.ends_with(ORIGINAL_SUFFIX) {
        // What does it mean to be an "original"?
        // "original" is an escape hatch for modded events that want to play a version that was defined in the "layer" above it.
        //
        // Let's say the event_string is `V_Critical_Original`.
        //
        // In the context of "Simple Event Replacement" (what's currently in the public release), it means that
        // 1. The person switch name is `PlayerF`
        // 2. The first time around, we found and played the modded event, `V_Critical_PlayerF`.
        // 3. However, the modded event's WEM file had a marker `CobaltEvent:V_Critical_Original`. Which
        //    means that we should play the original event, `V_Critical`, instead.
        // So this time on entering the function, we will skip looking for the modded event and play the original event instead.
        // There is no need to mess with the person switch name in this case.
        //
        // In the context of "Costume Voice", SeasideDragon!PlayerF
        //
        // 1. The person switch name that was used last time was `SeasideDragon`.
        // 2. The first time around, we found and played the modded event, `V_Critical_SeasideDragon`.
        // 3. However, the modded event's WEM file had a marker `CobaltEvent:V_Critical_Original`. Which
        //    means that we should play `V_Critical` again, but for the fallback person switch name of `PlayerF`.
        let stripped_name = event_string.trim_end_matches(ORIGINAL_SUFFIX);
        call_original!(
            gameobject,
            person_switch_name,
            engage_switch_name,
            stripped_name.into(),
            character,
            method_info
        );
        return;
    }

    let parsed_event = get_event_or_fallback(Il2CppString::new(format!("{}_{}", event_string, person_switch_name)));
    let parsed_switchname = get_switchname_fallback(person_switch_name);

    if unsafe { gamesound_iseventloaded(parsed_event, None) } {
        println!("[PersonVoice True]: Event name: {}, switch name: {}", parsed_event, parsed_switchname);
        call_original!(gameobject, parsed_switchname, engage_switch_name, parsed_event, character, method_info);
    } else {
        println!("[PersonVoice False]: Event name: {}, switch name: {}", event_string, parsed_switchname);
        call_original!(gameobject, parsed_switchname, engage_switch_name, event_name, character, method_info);
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

            match unsafe { gamesound_iseventloaded(modded_event, None) } {
                true => call_original!(side, person_switch_name, engage_switch_name, modded_event, method_info),
                false => call_original!(side, person_switch_name, engage_switch_name, event_name, method_info),
            }
        },

        _ => call_original!(side, person_switch_name, engage_switch_name, event_name, method_info),
    }
}

#[skyline::hook(offset = 0x2292F90)]
pub fn gamesound_ringcleaningvoice(person_switch_name: &Il2CppString, event_name: &Il2CppString, character: &Character, method_info: OptionalMethod) {
    let event_string = event_name.to_string();
    let person_string = person_switch_name.to_string();

    match event_string.contains("V_Ring") {
        true => {
            let modded_event = Il2CppString::new(format!("{}_{}", event_string, person_string));

            println!("[GameSound] {} => {}_{}", event_string, event_string, person_string);

            match unsafe { gamesound_iseventloaded(modded_event, None) } {
                true => call_original!(person_switch_name, modded_event, character, method_info),
                false => call_original!(person_switch_name, event_name, character, method_info),
            }
        },

        false => call_original!(person_switch_name, event_name, character, method_info),
    }
}

// App.GameSound$$LoadSystemVoice	710228e850	App_GameSound_ResultLoad_o * App.GameSound$$LoadSystemVoice(System_String_o * personSwitchName, MethodInfo * method)	544
#[unity::hook("App", "GameSound", "LoadSystemVoice")]
pub fn gamesound_loadsystemvoice(person_switch_name: &Il2CppString, method_info: OptionalMethod) -> &() {
    // match ParsedVoiceName::parse(person_switch_name) {
    //     ParsedVoiceName::ModdedVoiceName(mod_voice) => {
    //         println!(
    //             "Loading fallback system voice for {}, which is {}",
    //             person_switch_name, mod_voice.fallback_voice_name
    //         );
    //         call_original!(mod_voice.fallback_voice_name, method_info)
    //     },
    //     ParsedVoiceName::OriginalVoiceName(original_voice) => {
    //         println!("Loading original system voice for {}", original_voice.voice_name);
    //         call_original!(original_voice.voice_name, method_info)
    //     },
    // }
    let parsed_switchname = get_switchname_fallback(person_switch_name);
    call_original!(parsed_switchname, method_info)
}

// App.GameSound$$UnloadSystemVoice	710228ea70	void App.GameSound$$UnloadSystemVoice(System_String_o * personSwitchName, MethodInfo * method)	408
#[unity::hook("App", "GameSound", "UnloadSystemVoice")]
pub fn gamesound_unloadsystemvoice(person_switch_name: &Il2CppString, method_info: OptionalMethod) -> &() {
    // match ParsedVoiceName::parse(person_switch_name) {
    //     ParsedVoiceName::ModdedVoiceName(mod_voice) => {
    //         println!(
    //             "Unloading fallback system voice for {}, which is {}",
    //             person_switch_name, mod_voice.fallback_voice_name
    //         );
    //         call_original!(mod_voice.fallback_voice_name, method_info)
    //     },
    //     ParsedVoiceName::OriginalVoiceName(original_voice) => {
    //         println!("Unloading original system voice for {}", original_voice.voice_name);
    //         call_original!(original_voice.voice_name, method_info)
    //     },
    // }
    let parsed_switchname = get_switchname_fallback(person_switch_name);
    call_original!(parsed_switchname, method_info)
}

#[skyline::hook(offset = 0x24F8DE0)]
pub fn gamesound_setenumparam_gameobject(
    _this: &(),
    value_name: Option<&Il2CppString>,
    value: Option<&Il2CppString>,
    _game_object: &(),
    method_info: OptionalMethod,
) -> bool {
    let value_name_string = match value_name {
        Some(value_name_str) => value_name_str.to_string(),
        None => "None".to_string(),
    };

    let value_string = match value {
        Some(value_str) => value_str.to_string(),
        None => "None".to_string(),
    };

    // Try to not spam the console
    match value_name_string.as_str() {
        "Person" => {
            println!("[GameSound] value_name: {}", value_name_string);
            println!("[GameSound] value: {}", value_string);
        },
        _ => return call_original!(_this, value_name, value, _game_object, method_info),
    }

    // For current purposes, we don't need to do anything past this point if there's nothing to change.
    if value_string == "None" {
        return call_original!(_this, value_name, value, _game_object, method_info);
    }

    // Check category
    match value_name_string.as_str() {
        "Person" => {
            // Filter for '!' fallback
            let parsed_value = get_switchname_fallback(value.unwrap());
            if parsed_value == value.unwrap() {
                return call_original!(_this, value_name, value, _game_object, method_info);
            }
            println!("[GameSound] Parsed value: {}", parsed_value);
            return call_original!(_this, value_name, Some(parsed_value), _game_object, method_info);
        },
        _ => call_original!(_this, value_name, value, _game_object, method_info),
    }
}

#[skyline::hook(offset = 0x24f7550)]
pub fn soundmanager_postevent_with_temporarygameobject(
    this: &(),
    event_name: &Il2CppString,
    gameobject: Option<&()>,
    character: Option<&Character>,
    is_get_position: bool,
    method_info: OptionalMethod,
) -> *const u8 {
    let event_string = event_name.to_string();
    println!("[SoundManager] Event name: {}", event_string);

    match event_string.as_str() {
        "V_Pick" => {
            if unsafe { PLAY_ORIGINAL_V_PICK } {
                println!("Playing original V_Pick");
                unsafe {
                    PLAY_ORIGINAL_V_PICK = false;
                }
                return call_original!(this, event_name, gameobject, character, is_get_position, method_info);
            }

            let my_unit = MapMind::get_unit();
            let voice_name = unsafe { combat_character_appearance_create_for_sound(my_unit, None).sound.voice_id };

            println!("[SoundManager] Voice name: {}", voice_name);

            let voice_name_parsed = ParsedVoice::parse(voice_name);

            // TODO: Clean up these paths
            let modded_event = Il2CppString::new(format!("{}_{}", event_string, my_unit.get_person().get_ascii_name().unwrap()));
            println!("[SoundManager] TODO event: {}", modded_event);

            match unsafe { gamesound_iseventloaded(modded_event, None) } {
                true => {
                    println!("[SoundManager] TODO SUCCESS event: {}", modded_event);
                    call_original!(this, modded_event, gameobject, character, is_get_position, method_info)
                },
                false => {
                    println!("[SoundManager] TODO FAIL event: {}", event_string);
                    call_original!(this, event_name, gameobject, character, is_get_position, method_info)
                },
            }
        },
        _ => call_original!(this, event_name, gameobject, character, is_get_position, method_info),
    }
}

#[repr(i32)]
#[derive(PartialEq)]
pub enum AkCallbackType {
    EndOfEvent = 1,
    EndOfDynamicSequenceItem = 2,
    Marker = 4,
    Duration = 8,
    SpeakerVolumeMatrix = 16,
    Starvation = 32,
    MusicPlaylistSelect = 64,
    MusicPlayStarted = 128,
    MusicSyncBeat = 256,
    MusicSyncBar = 512,
    MusicSyncEntry = 1024,
    MusicSyncExit = 2048,
    MusicSyncGrid = 4096,
    MusicSyncUserCue = 8192,
    MusicSyncPoint = 16384,
    MusicSyncAll = 32512,
    MIDIEvent = 65536,
    CallbackBits = 1048575,
    EnableGetSourcePlayPosition = 1048576,
    EnableGetMusicPlayPosition = 2097152,
    EnableGetSourceStreamBuffering = 4194304,
    Monitoring = 536870912,
    Bank = 1073741824,
    AudioInterruption = 570425344,
    AudioSourceChange = 587202560,
}

pub static mut PLAY_ORIGINAL_V_PICK: bool = false;
pub const COBALT_EVENT_MARKER_PREFIX: &str = "CobaltEvent:";

// App.SoundWwise.SoundPlay$$PostEventCallback	71021f3a50	void App.SoundWwise.SoundPlay$$PostEventCallback(App_SoundWwise_SoundPlay_o * __this, Il2CppObject * cookie, int32_t type, AkCallbackInfo_o * callbackInfo, MethodInfo * method)	1760
#[skyline::hook(offset = 0x21f3a50)]
pub fn soundplay_posteventcallback(
    this: &(),
    cookie: &Il2CppObject<()>,
    callback_type: AkCallbackType,
    callback_info: &(),
    method_info: OptionalMethod,
) {
    if callback_type == AkCallbackType::Marker {
        let marker_label = unsafe { akmarkercallbackinfo_get_strlabel(callback_info, None) };
        // println!("[SoundPlay] Marker label: {}", marker_label.to_string());

        let label = marker_label.to_string();
        if label.starts_with(COBALT_EVENT_MARKER_PREFIX) {
            // println!("Entered CobaltEvent block");
            let event_name = label.trim_start_matches(COBALT_EVENT_MARKER_PREFIX);

            if event_name == "V_Pick_Original" {
                // println!("V_Pick_Original event detected");
                let my_unit = MapMind::get_unit();
                unsafe {
                    my_unit.last_pick_voice = PREVIOUS_LAST_PICK_VOICE;
                    PLAY_ORIGINAL_V_PICK = true;
                    gamesound_unitpickvoice(&my_unit, None);
                }
            } else {
                // other non-V_Pick events
                // we only support so-called "Original" events for now
                if event_name.ends_with(ORIGINAL_SUFFIX) {
                    // println!("Original event detected");
                    unsafe {
                        if !UNSAFE_CHARACTER_PTR.is_null() {
                            // Dereference the raw pointer to access the Character
                            let character_ref: &Character = &*UNSAFE_CHARACTER_PTR;
                            let sound = combat_character_get_sound(character_ref, None);
                            combat_charactersound_play_voice(sound, event_name.into(), None);
                            // Clear the pointer, as we're done with it
                            UNSAFE_CHARACTER_PTR = std::ptr::null();
                        }
                    }
                }
            }
        }
    }
    call_original!(this, cookie, callback_type, callback_info, method_info)
}

pub static mut PREVIOUS_LAST_PICK_VOICE: u8 = 0;
// App.GameSound$$UnitPickVoice	7102292360	void App.GameSound$$UnitPickVoice(App_Unit_o * unit, MethodInfo * method)	1012
#[unity::hook("App", "GameSound", "UnitPickVoice")]
pub fn gamesound_unitpickvoice(unit: &Unit, method_info: OptionalMethod) {
    // Save the last pick voice, which needs to be restored when playing an original V_Pick event
    // The actual restoration is done in the soundplay_posteventcallback hook
    unsafe { PREVIOUS_LAST_PICK_VOICE = unit.last_pick_voice }
    call_original!(unit, method_info);
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

// Careful, this is a nested class, do not instantiate this yourself
#[unity::class("App.SoundSystem", "SoundHandle")]
pub struct SoundHandle {}

impl SoundHandle {
    pub fn get_event_name(&self) -> &'static mut Il2CppString {
        self.get_class()
            .get_virtual_method("GetEventName")
            .map(|method| {
                let get_event_name = unsafe {
                    std::mem::transmute::<_, extern "C" fn(&Self, &MethodInfo) -> &'static mut Il2CppString>(method.method_info.method_ptr)
                };

                get_event_name(&self, method.method_info)
            })
            .unwrap()
    }
}

#[unity::class("App", "SoundManager")]
pub struct SoundManager {
    padding: [u8; 0x88],
    sound_handle_list: &'static List<SoundHandle>,
    // ...
}

impl SoundManager {
    pub fn is_event_playing_with_prefix<S: AsRef<str>>(&self, event_name: S, method_info: OptionalMethod) -> bool {
        let event_name = event_name.as_ref();

        self.sound_handle_list
            .iter()
            .find(|handle| handle.get_event_name().to_string().starts_with(event_name))
            .is_some()
    }
}

// App.SoundManager$$IsEventPlaying	71024f5520	bool App.SoundManager$$IsEventPlaying(App_SoundManager_o * __this, System_String_o * eventName, MethodInfo * method)	28
#[unity::hook("App", "SoundManager", "IsEventPlaying")]
pub fn soundmanager_iseventplaying(this: &SoundManager, event_name: Option<&Il2CppString>, method_info: OptionalMethod) -> bool {
    if let Some(name) = event_name {
        return this.is_event_playing_with_prefix(name.to_string(), method_info);
    }
    false
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

pub fn get_gid_from_ascii_name(ascii_name: &str) -> Option<&Il2CppString> {
    GodData::get_list()?.iter()
        .find(|data| data.get_ascii_name() == Some(ascii_name.into()))
        .map(|data| data.gid)
}

// allows for easy remapping of engage zones in the ring polish screen by utilizing the goddata's nickname field.
// makes it consistent with engage zone replacement in engage attacks.
#[skyline::hook(offset = 0x232E960)]
pub fn goddata_getengagezoneprefabpath(gid: &Il2CppString, method_info: OptionalMethod) -> &'static Il2CppString {
    let god_data = unsafe { god_pool_try_get_gid(gid, false, method_info) };
    match god_data {
        Some(god) => {
            let god_nickname = god.data.nickname.to_string();
            let stripped_identifier = match god_nickname.split('_').nth(2) {
                Some(identifier) => identifier,
                None => return call_original!(gid, method_info),
            };

            match get_gid_from_ascii_name(stripped_identifier) {
                Some(remapped_gid) => {
                    if gid == remapped_gid {
                        return call_original!(gid, method_info);
                    }

                    println!("[EngageZone] {} => {}", gid, remapped_gid);
                    call_original!(remapped_gid, method_info)
                }
                _ => call_original!(gid, method_info),
            }
        },
        None => call_original!(gid, method_info),
    }
}