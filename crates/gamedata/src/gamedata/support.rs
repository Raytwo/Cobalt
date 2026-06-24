use astra_derive::{Astra, AstraBook};
use astra_formats::{indexmap::IndexMap, Sheet, SheetHeader};
use differ::{Differ, Tag};

#[derive(AstraBook, Clone)]
pub struct SupportBook {
    pub sets: Sheet<IndexMap<String, Vec<SupportSet>>>,
}

impl SupportBook {
    pub fn new() -> Self {
        SupportBook {
            sets: Sheet {
                name: String::from("Support"),
                header: SheetHeader {
                    params: vec![]
                },
                data: IndexMap::new(),
            }
        }
    }
}

impl Default for SupportBook {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Astra, Debug, Eq, PartialEq, Hash, Clone)]
pub struct SupportSet {
    #[astra(key = "@Condition", public_array)]
    pub condition: String,
    #[astra(key = "@Pid")]
    pub pid: String,
    #[astra(key = "@ExpType")]
    pub exp_type: Option<u8>,
}

impl super::XmlPatch for SupportBook {
    fn patch(&mut self, patch: Self, original: &Self) {
        for (pid, supports) in patch.sets.data {
            // Can eventually finish with an empty group
            if supports.is_empty() {
                continue;
            }

            // Grab the original group for comparisons
            let original_group = original.sets.data.get(&pid);

            // Get or insert the current group in the patched book
            let current_group = self.sets.data.entry(pid).or_default();

            // Compare to the original group if it exists, or the patched one if it does not
            let differ = Differ::new(original_group.unwrap_or(current_group), &supports);

            // Process in reverse order so insertions/deletions don't shift the indices
            for span in differ.spans().iter().rev() {
                match span.tag {
                    Tag::Insert => {
                        for i in (span.b_start..span.b_end).rev() {
                            current_group.insert(span.a_start, supports[i].clone())
                        }
                    },
                    Tag::Replace => {
                        current_group.splice(span.a_start..span.a_end, supports[span.b_start..span.b_end].iter().cloned());
                    },
                    Tag::Delete => {
                        for i in (span.a_start..span.a_end).rev() {
                            current_group.remove(i);
                        }
                    },
                    _ => (),
                }
            }
        }
    }
}
