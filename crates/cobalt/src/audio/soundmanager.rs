use engage::{combat::Character, gamesound::GameSound, mapmind::MapMind, soundmanager::SoundManager};
use unity::prelude::*;

use crate::audio::{combat_character_appearance_create_for_sound, ParsedVoice, PLAY_ORIGINAL_V_PICK};

// App.SoundManager$$IsEventPlaying	71024f5520	bool App.SoundManager$$IsEventPlaying(App_SoundManager_o * __this, System_String_o * eventName, MethodInfo * method)	28
#[unity::hook("App", "SoundManager", "IsEventPlaying")]
pub fn soundmanager_iseventplaying(this: &SoundManager, event_name: Option<&Il2CppString>, _method_info: OptionalMethod) -> bool {
    event_name.map(|name| this.is_event_playing_with_prefix(name.to_string())).unwrap_or_default()
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
            if PLAY_ORIGINAL_V_PICK.load(std::sync::atomic::Ordering::Relaxed) {
                println!("Playing original V_Pick");
                PLAY_ORIGINAL_V_PICK.store(false, std::sync::atomic::Ordering::Relaxed);
                return call_original!(this, event_name, gameobject, character, is_get_position, method_info);
            }

            let my_unit = MapMind::get_unit();
            // Read the `Voice` value. This corresponds to the `Voice` set via the AssetTable.
            let voice_name = unsafe { combat_character_appearance_create_for_sound(my_unit, None).sound.voice_id };

            // Check if the voice_name is a special one that defines a main voice and a fallback like `Voice="SeasideDragon!PlayerF"``
            let voice_name_parsed = ParsedVoice::parse(voice_name);

            match ParsedVoice::parse(voice_name) {
                ParsedVoice::DefaultVoiceEvent(_) => {
                   // do nothing - legacy behavior
                },
                ParsedVoice::ModdedVoiceEvent(modded_voice) => {
                    let modded_event = Il2CppString::new(format!("{}_{}", event_string, modded_voice.mod_event));
                    // Check if we have a loaded custom V_Pick, that looks like V_Pick_<unit_name>, for example V_Pick_SeasideDragon
                    // if we do, play it.
                    if GameSound::is_event_loaded(modded_event) {
                        println!("[SoundManager] Found modded event: {}", modded_event);
                        return call_original!(this, modded_event, gameobject, character, is_get_position, method_info);
                    }
                    // Attempt to use the fallback event if the custom V_Pick is not loaded
                    // For example, this will support a V_Pick_PlayerF event that perhaps other mod might add.
                    let fallback_event = Il2CppString::new(format!("{}_{}", event_string, modded_voice.fallback_event));
                    if GameSound::is_event_loaded(fallback_event) {
                        println!("[SoundManager] Found fallback event: {}", fallback_event);
                        return call_original!(this, fallback_event, gameobject, character, is_get_position, method_info);
                    }
                },
            }

            // Try to find an ASCII name fallback for the character. If it exists, use it.
            let modded_event = Il2CppString::new(format!("{}_{}", event_string, my_unit.get_person().get_ascii_name().unwrap()));
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