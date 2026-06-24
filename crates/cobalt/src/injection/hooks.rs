use std::ffi::c_char;

use skyline::hook;
use unity::il2cpp::Il2CppClass;

use super::registry;

#[hook(offset = 0x5919b4)]
pub unsafe fn mono_class_from_assembly_namespace_name_hook(
    state: u64,
    assembly: *const c_char,
    namespace: *const c_char,
    name: *const c_char,
) -> Option<&'static mut Il2CppClass> {
    if let Some(class) = registry::lookup_by_name_raw(namespace as *const u8, name as *const u8) {
        return Some(class);
    }
    call_original!(state, assembly, namespace, name)
}
