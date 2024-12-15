mod support;
mod vibrationevent;
pub use support::*;
pub use vibrationevent::*;

pub trait XmlPatch {
    fn patch(&mut self, patch: Self, original: &Self);
}
