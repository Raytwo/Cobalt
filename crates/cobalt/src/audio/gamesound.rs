use engage::combat::character::Character;
use engage::app::unit::{IUnitMethods, Unit};
use engage::app::gamesound::GameSound;
use unity::prelude::*;

use crate::audio::{
    get_event_or_fallback, get_switchname_fallback, ORIGINAL_SUFFIX, PREVIOUS_LAST_PICK_VOICE,
    UNSAFE_CHARACTER_PTR,
};

pub struct GameSoundStatic {
    pub default_bank_name_array: Array<Il2CppString>,
}

#[skyline::hook(offset = 0x2287c60)]
pub fn load_default_sound_banks(method_info: OptionalMethod) {
    let static_fields =
        unsafe { &mut *(GameSound::class().raw().static_fields as *mut GameSoundStatic) };

    let mut banks: Vec<Il2CppString> = static_fields.default_bank_name_array.iter().collect();

    for path in mods::list_recursive("Data/StreamingAssets/Audio/GeneratedSoundBanks/Switch")
        .filter(|path| path.extension() == Some("bnk"))
    {
        let filename: Il2CppString = path.file_stem().unwrap().into();

        if !banks.contains(&filename) {
            println!("Added '{}' to the list", path);
            banks.insert(0, filename);
        }
    }

    static_fields.default_bank_name_array = Array::<Il2CppString>::from_slice(&banks).unwrap();

    call_original!(method_info);
}

#[skyline::hook(offset = 0x2292270)]
pub fn gamesound_personvoice(
    gameobject: IlInstance,
    person_switch_name: Il2CppString,
    engage_switch_name: Il2CppString,
    event_name: Il2CppString,
    character: Character,
    method_info: OptionalMethod,
) {
    let event_string = event_name.to_string();
    // println!(
    //     "[PersonVoice]: Event name: {}, switch name: {}",
    //     event_string,
    //     person_switch_name
    // );

    unsafe {
        UNSAFE_CHARACTER_PTR = character.as_instance().as_ptr();
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

    let parsed_event = get_event_or_fallback(format!("{}_{}", event_string, person_switch_name).as_str().into());
    let parsed_switchname = get_switchname_fallback(person_switch_name);

    if GameSound::is_event_loaded(parsed_event) {
        println!("[PersonVoice True]: Event name: {}, switch name: {}", parsed_event, parsed_switchname);
        call_original!(gameobject, parsed_switchname, engage_switch_name, parsed_event, character, method_info);
    } else {
        println!("[PersonVoice False]: Event name: {}, switch name: {}", event_string, parsed_switchname);
        call_original!(gameobject, parsed_switchname, engage_switch_name, event_name, character, method_info);
    }
}

#[skyline::hook(offset = 0x2292F90)]
pub fn gamesound_ringcleaningvoice(
    person_switch_name: Il2CppString,
    event_name: Il2CppString,
    character: Character,
    method_info: OptionalMethod,
) {
    let event_string = event_name.to_string();
    let person_string = person_switch_name.to_string();

    match event_string.contains("V_Ring") {
        true => {
            let modded_event: Il2CppString = format!("{}_{}", event_string, person_string).as_str().into();

            println!("[GameSound] {} => {}_{}", event_string, event_string, person_string);

            match GameSound::is_event_loaded(modded_event) {
                true => call_original!(person_switch_name, modded_event, character, method_info),
                false => call_original!(person_switch_name, event_name, character, method_info),
            }
        },

        false => call_original!(person_switch_name, event_name, character, method_info),
    }
}

// App.GameSound$$LoadSystemVoice	710228e850	App_GameSound_ResultLoad_o * App.GameSound$$LoadSystemVoice(System_String_o * personSwitchName, MethodInfo * method)	544
#[unity::hook("App", "GameSound", "LoadSystemVoice")]
pub fn gamesound_loadsystemvoice(person_switch_name: Il2CppString, method_info: OptionalMethod) -> IlInstance {
    let parsed_switchname = get_switchname_fallback(person_switch_name);
    call_original!(parsed_switchname, method_info)
}

// App.GameSound$$UnloadSystemVoice	710228ea70	void App.GameSound$$UnloadSystemVoice(System_String_o * personSwitchName, MethodInfo * method)	408
#[unity::hook("App", "GameSound", "UnloadSystemVoice")]
pub fn gamesound_unloadsystemvoice(person_switch_name: Il2CppString, method_info: OptionalMethod) {
    let parsed_switchname = get_switchname_fallback(person_switch_name);
    call_original!(parsed_switchname, method_info)
}

#[skyline::hook(offset = 0x24F8DE0)]
pub fn gamesound_setenumparam_gameobject(
    this: IlInstance,
    value_name: Il2CppString,
    value: Il2CppString,
    game_object: IlInstance,
    method_info: OptionalMethod,
) -> bool {
    let value_name_string = if value_name.is_null() { "None".to_string() } else { value_name.to_string() };
    let value_string = if value.is_null() { "None".to_string() } else { value.to_string() };

    // Try to not spam the console
    match value_name_string.as_str() {
        "Person" => {
            println!("[GameSound] value_name: {}", value_name_string);
            println!("[GameSound] value: {}", value_string);
        },
        _ => return call_original!(this, value_name, value, game_object, method_info),
    }

    // For current purposes, we don't need to do anything past this point if there's nothing to change.
    if value_string == "None" {
        return call_original!(this, value_name, value, game_object, method_info);
    }

    // Check category
    match value_name_string.as_str() {
        "Person" => {
            // Filter for '!' fallback
            let parsed_value = get_switchname_fallback(value);
            if parsed_value == value {
                return call_original!(this, value_name, value, game_object, method_info);
            }
            println!("[GameSound] Parsed value: {}", parsed_value);
            call_original!(this, value_name, parsed_value, game_object, method_info)
        },
        _ => call_original!(this, value_name, value, game_object, method_info),
    }
}

// App.GameSound$$UnitPickVoice	7102292360	void App.GameSound$$UnitPickVoice(App_Unit_o * unit, MethodInfo * method)	1012
    // This looks pretty insane, and it is.
    //
    // Each unit has a set of voice clips that they can play when you select them.
    // They vary based on the unit's current health status (High, Medium, Low).
    //
    // The game keeps track of which type of voice clip was last played for a unit.
    // For example, if the unit is at High health, and you select them, they will say a line from the High pool. 
    // However, if you select them again, they won't say any High line again, and instead, remain silent.
    //
    // Then later, if they take damage, and you select them, they will say a Medium or Low line.
    //
    // You won't hear a V_Pick line again until their health status changes.
    //
    // The last_pick_voice field is used to keep track of the last voice clip type that was played.
    //
    // We need to save the last pick voice, and potentially later restore it when playing an original V_Pick event.
    //
    // If we don't, then the game thinks that the unit has already played the voice clip, and won't play it again.
    //
    // The actual restoration is done in the soundplay_posteventcallback hook
#[unity::hook("App", "GameSound", "UnitPickVoice")]
pub fn gamesound_unitpickvoice(unit: Unit, method_info: OptionalMethod) {
    unsafe { PREVIOUS_LAST_PICK_VOICE = unit.get_last_pick_voice() }
    call_original!(unit, method_info);
}
