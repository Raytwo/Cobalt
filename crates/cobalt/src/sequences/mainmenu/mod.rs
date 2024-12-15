use unity::prelude::*;

use engage::{
    menu::MenuSequence,
    proc::{desc::ProcDesc, ProcVoidFunction, ProcVoidMethod},
    sequence::mainmenusequence::{
        mainmenusequence_jumptonextsequence,
        MainMenuSequence
    },
};

use crate::{updater::update_check, sequences::mainmenu::cobaltmenu::CobaltMenuSequence};

pub mod cobaltmenu;
pub mod topmenu;

#[skyline::hook(offset = 0x1bf7a00)]
pub fn mainmenusequence_getdesc_hook(
    proc: &'static MainMenuSequence,
    method_info: OptionalMethod,
) -> &'static mut Il2CppArray<&'static mut ProcDesc> {
    // Get the original ProcDescs for the MainMenuSequence
    let descs = call_original!(proc, method_info);

    // Turn the array into a Vec so we can manipulate it more simply
    let mut vec = descs.to_vec();

    let method = mainmenusequence_jumptonextsequence::get_ref();
    let method = unsafe { std::mem::transmute(method.method_ptr) };
    
    let desc = ProcDesc::call(ProcVoidMethod::new(proc, method));

    // Inject in reverse order so future injections don't get shifted

    // JumpToNextSequence call
    vec.insert(0x62, desc);
    
    let desc = ProcDesc::call(ProcVoidFunction::new(proc, mainmenusequence_createcobaltmenu));
    vec.insert(0x62, desc);
    // Made up Label behind every existing one so we don't conflict with the order of things
    let custom_label = ProcDesc::label(31);
    vec.insert(0x62, custom_label);

    // Prepare a ProcDesc Method Call to check for updates
    let desc = ProcDesc::call(ProcVoidMethod::new(proc, update_check));
    // Inject our update check between the call to Start and the DLC News check
    vec.insert(1, desc);

    // Turn the vector into a Il2CppArray and return it
    Il2CppArray::from_slice(vec).unwrap()
}

pub extern "C" fn mainmenusequence_createcobaltmenu(target: &'static mut MainMenuSequence, _method_info: OptionalMethod) {
    println!("MainMenuSequence::CreateCobaltMenu");
    CobaltMenuSequence::bind(target);
    target.next_sequence = 0x13;
}
