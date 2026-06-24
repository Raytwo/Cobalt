use std::collections::HashMap;
use std::ffi::{CStr, c_char};
use std::sync::{LazyLock, RwLock};

use unity::Class;
use unity::il2cpp::{Il2CppClass, Il2CppType, MethodInfo};

pub struct InjectedEntry {
    pub class: Class,
    pub parent_ctor: Option<&'static MethodInfo>,
}

#[derive(Default)]
struct Registry {
    // namespace -> name -> entry, nested so a lookup can probe with &str at
    // both levels and never allocate a key
    by_name: HashMap<String, HashMap<String, InjectedEntry>>,
    by_type: HashMap<usize, *const Il2CppClass>,
}

unsafe impl Send for Registry {}
unsafe impl Sync for Registry {}

static REGISTRY: LazyLock<RwLock<Registry>> = LazyLock::new(|| RwLock::new(Registry::default()));

pub fn insert(
    namespace: &'static str,
    name: &'static str,
    class: Class,
    parent_ctor: Option<&'static MethodInfo>,
) {
    let mut g = REGISTRY.write().expect("injection registry poisoned");
    let byval_arg_ptr = &class.raw()._1.byval_arg as *const Il2CppType as usize;
    g.by_type.insert(byval_arg_ptr, class.raw() as *const Il2CppClass);
    g.by_name
        .entry(namespace.to_string())
        .or_default()
        .insert(name.to_string(), InjectedEntry { class, parent_ctor });
}

pub fn insert_raw(class: *mut Il2CppClass) -> bool {
    if class.is_null() {
        return false;
    }
    let class = Class::from_raw(unsafe { &*class });
    let namespace = class.namespace();
    let name = class.name();
    let parent_ctor: Option<&'static MethodInfo> =
        class.raw().get_method_from_name(".ctor", 0).map(|mi| &*mi);

    let mut g = REGISTRY.write().expect("injection registry poisoned");
    if g.by_name.get(&namespace).is_some_and(|inner| inner.contains_key(&name)) {
        return false;
    }

    let byval_arg_ptr = &class.raw()._1.byval_arg as *const Il2CppType as usize;
    g.by_type.insert(byval_arg_ptr, class.raw() as *const Il2CppClass);
    g.by_name
        .entry(namespace)
        .or_default()
        .insert(name, InjectedEntry { class, parent_ctor });
    true
}

pub fn lookup_by_name(namespace: &str, name: &str) -> Option<&'static mut Il2CppClass> {
    let g = REGISTRY.read().expect("injection registry poisoned");
    let entry = g.by_name.get(namespace)?.get(name)?;
    Some(entry.class.raw_mut())
}

// Shared by the class_from_name forwarder and the game-side mono hook so
// both resolve injected classes the same way. Returns None for null or
// non-UTF8 names, which just means "fall through to the real lookup".
pub fn lookup_by_name_raw(namespace: *const u8, name: *const u8) -> Option<&'static mut Il2CppClass> {
    if namespace.is_null() || name.is_null() {
        return None;
    }
    let ns = unsafe { CStr::from_ptr(namespace as *const c_char) }.to_str().ok()?;
    let n = unsafe { CStr::from_ptr(name as *const c_char) }.to_str().ok()?;
    lookup_by_name(ns, n)
}

pub fn lookup_by_type(ty: *const Il2CppType) -> Option<&'static mut Il2CppClass> {
    let g = REGISTRY.read().expect("injection registry poisoned");
    let class_ptr = *g.by_type.get(&(ty as usize))?;
    #[allow(invalid_reference_casting)]
    Some(unsafe { &mut *(class_ptr as *mut Il2CppClass) })
}

pub fn parent_ctor(class: Class) -> Option<&'static MethodInfo> {
    let g = REGISTRY.read().expect("injection registry poisoned");
    g.by_name
        .values()
        .flat_map(|inner| inner.values())
        .find(|e| e.class == class)
        .and_then(|e| e.parent_ctor)
}
