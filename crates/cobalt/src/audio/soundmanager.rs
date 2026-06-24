use engage::app::gamesound::GameSound;
use engage::app::mapmind::{IMapMindMethods, MapMind};
use engage::app::persondata::IPersonDataMethods;
use engage::app::singletonclass_1::ISingletonClass_1Methods;
use engage::app::soundmanager::SoundManager;
use engage::app::unit::IUnitMethods;
use engage::combat::character::Character;
use engage::combat::characterappearance::{CharacterAppearance, ICharacterAppearance};
use unity::prelude::*;

use engage::SoundManagerExt;

use crate::audio::{ParsedVoice, PLAY_ORIGINAL_V_PICK};

#[unity::hook("App", "SoundManager", "IsEventPlaying")]
pub fn soundmanager_iseventplaying(this: SoundManager, event_name: Il2CppString, _method_info: OptionalMethod) -> bool {
    if event_name.is_null() {
        false
    } else {
        this.is_event_playing_with_prefix(event_name.to_string())
    }
}

// This hook is responsible for playing custom V_Pick events.
//
/// Handling "Original" V_Pick sounds
//
// This is for letting a custom soundbank reuse the game's original V_Pick sounds.
//
// First, we check if there was an request to play an "Original" V_Pick sound.
// If so, we clear the flag, and fire the V_Pick event, without any manipulation to it.
//
// While it might seem like this won't work since our unit has a custom voice name like "SeasideDragon!PlayerF",
// due to how we patched the function `gamesound_setenumparam_gameobject`,
// the fallback name (for example, "PlayerF") has already been set in the Person switch for us, and thus WWise will play the default V_Pick event with the fallback name.
//
// The end behavior is as if we were never here, and the game will play the default V_Pick event.
//
//
/// Handling custom V_Pick sounds
//
// First, we get the unit that is currently being selected.
// Then, we get the voice field of the unit.
//
// If it's a custom voice line (for example, "SeasideDragon!PlayerF"), we check if the custom V_Pick event is loaded (for example, "V_Pick_SeasideDragon").
// If it is, we play it.
//
// If it's not, we check if the fallback event is loaded (for example, "V_Pick_PlayerF").
// If it is, we play it.
//
// If neither the custom event nor the fallback event is loaded, we try to play the V_Pick_<ascii_name> event.
// If it is loaded, we play it.
//
// If none of the above events are loaded, we play the default V_Pick event.
#[skyline::hook(offset = 0x24f7550)]
pub fn soundmanager_postevent_with_temporarygameobject(
    this: IlInstance,
    event_name: Il2CppString,
    gameobject: IlInstance,
    character: Character,
    is_get_position: bool,
    method_info: OptionalMethod,
) -> *const u8 {
    let event_string = event_name.to_string();
    println!("[SoundManager] Event name: {}", event_string);

    match event_string.as_str() {
        "V_Pick" => {
            if PLAY_ORIGINAL_V_PICK.load(std::sync::atomic::Ordering::Relaxed) {
                println!("Playing original V_Pick");
                PLAY_ORIGINAL_V_PICK.store(false, std::sync::atomic::Ordering::Relaxed);
                return call_original!(this, event_name, gameobject, character, is_get_position, method_info);
            }

            let my_unit = MapMind::get_instance().get_unit();
            if my_unit.is_null() {
                return call_original!(this, event_name, gameobject, character, is_get_position, method_info);
            }
            let person = my_unit.get_person();
            if person.is_null() {
                return call_original!(this, event_name, gameobject, character, is_get_position, method_info);
            }

            let appearance = CharacterAppearance::create_for_sound(my_unit);
            let voice_name = appearance.sound().voice_id;

            match ParsedVoice::parse(voice_name) {
                ParsedVoice::DefaultVoiceEvent(_) => {},
                ParsedVoice::ModdedVoiceEvent(modded_voice) => {
                    let modded_event: Il2CppString =
                        format!("{}_{}", event_string, modded_voice.mod_event).as_str().into();
                    if GameSound::is_event_loaded(modded_event) {
                        println!("[SoundManager] Found modded event: {}", modded_event);
                        return call_original!(this, modded_event, gameobject, character, is_get_position, method_info);
                    }
                    let fallback_event: Il2CppString =
                        format!("{}_{}", event_string, modded_voice.fallback_event).as_str().into();
                    if GameSound::is_event_loaded(fallback_event) {
                        println!("[SoundManager] Found fallback event: {}", fallback_event);
                        return call_original!(this, fallback_event, gameobject, character, is_get_position, method_info);
                    }
                },
            }

            let modded_event: Il2CppString =
                format!("{}_{}", event_string, person.get_ascii_name()).as_str().into();
            println!("[SoundManager] Trying V_Pick_<ascii_name> event: {}", modded_event);

            let name = if GameSound::is_event_loaded(modded_event) {
                println!("[SoundManager] <ascii_name> event: {}", modded_event);
                modded_event
            } else {
                println!("[SoundManager] Default V_Pick event: {}", event_string);
                event_name
            };

            call_original!(this, name, gameobject, character, is_get_position, method_info)
        },
        _ => call_original!(this, event_name, gameobject, character, is_get_position, method_info),
    }
}
