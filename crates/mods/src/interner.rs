use std::{
    convert::TryInto,
    num::NonZeroU16,
    path::{Component, Path},
};

use camino::{ Utf8Path, Utf8PathBuf };
use std::collections::HashMap;

#[derive(Default)]
pub struct Interner {
    strings: Vec<String>,
}

#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct StrId(NonZeroU16);

#[repr(transparent)]
pub struct InternedPath<const MAX_COMPONENTS: usize>([Option<StrId>; MAX_COMPONENTS]);

impl Interner {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn get(&self, id: StrId) -> &str {
        self.strings.get(id.0.get().saturating_sub(1) as usize).unwrap()
    }

    pub fn add(&mut self, string: String) -> StrId {
        let idx = if let Some(existing_string) = self.strings.iter().position(|s| *s == string) {
            existing_string + 1
        } else {
            self.strings.push(string);

            self.strings.len()
        };

        StrId(NonZeroU16::new(idx.try_into().unwrap()).unwrap())
    }

    pub fn add_path<const N: usize>(&mut self, path: &Path) -> InternedPath<N> {
        let component_count = path.components().count();
        if component_count > N {
            panic!("Path has {} components, only a max of {} are allowed.", component_count, N);
        }

        let components = path.components().filter_map(|component| {
            if let Component::Normal(component) = component {
                Some(component.to_string_lossy().into_owned())
            } else {
                None
            }
        });

        let mut path = InternedPath([None; N]);

        for (i, component) in components.enumerate() {
            path.0[i].replace(self.add(component));
        }

        path
    }
}

impl<const N: usize> InternedPath<N> {
    pub fn to_string(&self, interner: &Interner) -> String {
        let slashes = self.components(interner).count().saturating_sub(1);
        let length = self.components(interner).map(|c| c.len()).sum::<usize>() + slashes;

        let mut string = String::with_capacity(length);
        let mut comps = self.components(interner);

        if let Some(first_comp) = comps.next() {
            string.push_str(first_comp);
        }

        for component in comps {
            string.push('/');
            string.push_str(component);
        }

        string
    }

    pub fn to_utf8pathbuf(&self, interner: &Interner) -> Utf8PathBuf {
        Utf8PathBuf::from(self.to_string(interner))
    }

    pub fn components<'a>(&'a self, interner: &'a Interner) -> impl Iterator<Item = &'a str> + 'a {
        self.0.iter().filter_map(move |c| c.map(|comp| interner.get(comp)))
    }
}

pub const MAX_COMPONENT_COUNT: usize = 10;

pub struct HashedPathInterner<const MAX_COMPONENTS: usize> {
    interner: Interner,
    hashes: HashMap<u64, InternedPath<MAX_COMPONENTS>>
}

impl<const MAX_COMPONENTS: usize> HashedPathInterner<MAX_COMPONENTS> {
    pub fn new() -> Self {
        Self {
            interner: Interner::new(),
            hashes: HashMap::new(),
        }
    }

    pub fn paths(&self) -> impl Iterator<Item = Utf8PathBuf> + '_ {
        self.hashes.values().map(|interned| interned.to_utf8pathbuf(&self.interner))
    }

    pub fn add<H: Into<u64>, P: AsRef<Utf8Path>>(&mut self, hash: H, new_path: P) {
        let path = new_path.as_ref();
        let interned = self.interner.add_path(path.as_std_path());

        self.hashes.insert(hash.into(), interned);
    }

    pub fn try_get<H: Into<u64>>(&self, hash: H) -> Option<Utf8PathBuf> {
        self.hashes.get(&hash.into()).map(|interned| interned.to_utf8pathbuf(&self.interner))
    }

    pub fn contains_key<H: Into<u64>>(&self, hash: H) -> bool {
        self.hashes.contains_key(&hash.into())
    }
}

impl<const MAX_COMPONENTS: usize> Default for HashedPathInterner<MAX_COMPONENTS> {
    fn default() -> Self {
        Self::new()
    }
}