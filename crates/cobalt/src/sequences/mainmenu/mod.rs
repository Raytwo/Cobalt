use engage::app::mainmenusequence::{IMainMenuSequence, MainMenuSequence, MainMenuSequence_Label};
use engage::app::proc::Proc;
use engage::app::procdesc::ProcDesc;
use engage::app::procinst::ProcInst;
use engage::app::procvoidfunction::ProcVoidFunction;
use engage::app::procvoidmethod::ProcVoidMethod;
use unity::prelude::*;
use unity::{ClassIdentity, FromIlInstance};

use engage::{ProcDescPatch, ProcExt, ProcVoidFunctionExt, ProcVoidMethodExt};

use crate::sequences::mainmenu::cobaltmenu::CobaltMenuSequence;
use crate::updater::update_check;

pub mod cobaltmenu;
pub mod topmenu;

#[skyline::hook(offset = 0x1bf7a00)]
pub fn mainmenusequence_getdesc_hook(
    proc: MainMenuSequence,
    method_info: OptionalMethod,
) -> Array<ProcDesc> {
    let jump_mi = MainMenuSequence::class()
        .raw()
        .get_method_from_name("JumpToNextSequence", 0)
        .expect("MainMenuSequence::JumpToNextSequence method missing");

    let jump_method = ProcVoidMethod::from_raw_parts(proc.as_instance(), jump_mi)
        .expect("ProcVoidMethod for JumpToNextSequence failed");

    let create_fn = ProcVoidFunction::from_fn(mainmenusequence_createcobaltmenu)
        .expect("ProcVoidFunction for CreateCobaltMenu failed");

    let update_method = ProcVoidMethod::from_fn(proc.into(), update_check)
        .expect("ProcVoidMethod for update_check failed");
    
    ProcDescPatch::new(call_original!(proc, method_info))
        .insert(1, [Proc::call_method(update_method)])
        // Custom label 31 sits behind every vanilla label so existing menus don't collide
        .insert(
            0x62,
            [
                Proc::label(31),
                Proc::call_function(create_fn),
                Proc::call_method(jump_method),
            ],
        )
        .finish()
}

pub extern "C" fn mainmenusequence_createcobaltmenu(inst: ProcInst, _method_info: OptionalMethod) {
    println!("MainMenuSequence::CreateCobaltMenu");
    CobaltMenuSequence::bind(inst);

    let mms = MainMenuSequence::from_il_instance(inst.as_instance());
    mms.set_m_next_sequence(MainMenuSequence_Label { value: 0x13 });
}
