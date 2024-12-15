use engage::{
    proc::desc::{
        ProcDesc, ProcDescType
    },
    sequence::titleloopsequence::TitleLoopSequenceLabel,
};

use crate::api::events::{Event, SystemEvent};

const TITLELOOPSEQUENCE_HASHCODE: i32 = -988690862;

pub extern "C" fn grand_opening_skip(evt: &Event<SystemEvent>) {
    if let Event::Args(ev) = evt {
        if let SystemEvent::ProcInstJump { proc, .. } = ev {
            // TitleLoopSequence
            if proc.hashcode == TITLELOOPSEQUENCE_HASHCODE {
                // TODO: Move this to a global save flag
                if std::path::Path::new("sd:/engage/config/gop_skip").exists() {
                    if let Some(jump) = proc
                        .get_descs_mut()
                        .iter_mut()
                        .find(|desc| desc.get_desc_type() == ProcDescType::Jump && desc.get_label() == TitleLoopSequenceLabel::GrandOpening as i32) {
                            *jump = ProcDesc::jump(TitleLoopSequenceLabel::Title as _);
                        }
                }
            }
        }
    }
}