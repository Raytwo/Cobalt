use engage::app::proc::Proc;
use engage::app::procdesc::{IProcDescMethods, ProcDesc, ProcDesc_Type};
use engage::app::procinst::IProcInst;
use engage::app::titleloopsequence::TitleLoopSequence_Label;

use crate::api::events::{Event, SystemEvent};

pub const TITLELOOPSEQUENCE_HASHCODE: i32 = -988690862;

fn proc_desc_label(desc: ProcDesc) -> i32 {
    let class = unity::object_get_class(desc);
    let entry = class
        .get_virtual_method("get_Label")
        .expect("ProcDesc vtable missing `get_Label`");
    let f: extern "C" fn(ProcDesc, &'static unity::MethodInfo) -> i32 =
        unsafe { std::mem::transmute(entry.method_ptr) };
    f(desc, entry.method_info)
}

pub extern "C" fn grand_opening_skip(evt: &Event<SystemEvent>) {
    let Event::Args(SystemEvent::ProcInstJump { proc, .. }) = evt else {
        return;
    };

    if proc.m_hash_code() != TITLELOOPSEQUENCE_HASHCODE {
        return;
    }

    if !std::path::Path::new("sd:/engage/config/gop_skip").exists() {
        return;
    }

    let descs = proc.m_descs();
    for i in 0..descs.len() {
        let desc = descs.get(i);
        if desc.get_desc_type() == ProcDesc_Type::jump()
            && proc_desc_label(desc) == TitleLoopSequence_Label::grand_opening().value
        {
            descs.set(i, Proc::jump(TitleLoopSequence_Label::title().value));
            break;
        }
    }
}
