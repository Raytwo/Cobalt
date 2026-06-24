//! Wrappers for the il2cpp_* runtime functions.
//!
//! Unity doesn't export these in the game binary, so plugins have to
//! scan the game's code to find them. The problem: when Cobalt installs
//! a hook on one, the first few bytes of that function change and the
//! scan in plugins no longer matches.
//!
//! Fix: Cobalt scans them once at startup and re-exports them as named
//! symbols (il2cpp_array_new and so on). Plugins call them like any
//! normal extern function and skyline wires it up at plugin load time,
//! same way cobapi_* already works today.
//!
//! init() has to run before any skyline hook is installed, otherwise
//! our own scan trips over the patched bytes.

use std::sync::OnceLock;

use skyline::hooks::{getRegionAddress, Region};

pub mod forwarders;

// Empty marker structs, we only pass pointers through. Each crate that
// uses these keeps its own real definition with the fields
#[repr(C)]
pub struct Il2CppImage          { _opaque: [u8; 0] }
#[repr(C)]
pub struct Il2CppClass          { _opaque: [u8; 0] }
#[repr(C)]
pub struct Il2CppArray          { _opaque: [u8; 0] }
#[repr(C)]
pub struct Il2CppObject         { _opaque: [u8; 0] }
#[repr(C)]
pub struct Il2CppString         { _opaque: [u8; 0] }
#[repr(C)]
pub struct Il2CppType           { _opaque: [u8; 0] }
#[repr(C)]
pub struct Il2CppReflectionType { _opaque: [u8; 0] }
#[repr(C)]
pub struct MethodInfo           { _opaque: [u8; 0] }

// One alias per function so the slot, the forwarder body, and any
// downstream extern decl all change together if a signature shifts
pub type Il2CppInitFn                   = unsafe extern "C" fn(domain_name: *const i8) -> i32;
pub type Il2CppAssemblyGetImageFn       = unsafe extern "C" fn(c_str: *const u8) -> *const Il2CppImage;
pub type Il2CppClassFromNameFn          = unsafe extern "C" fn(image: *const Il2CppImage, namespace: *const u8, name: *const u8) -> *mut Il2CppClass;
pub type Il2CppObjectNewFn              = unsafe extern "C" fn(klass: *const Il2CppClass) -> *mut Il2CppObject;
pub type Il2CppClassGetMethodFromNameFn = unsafe extern "C" fn(class: *const Il2CppClass, method_name: *const u8, args_count: usize, flags: u32) -> *mut MethodInfo;
pub type Il2CppArrayNewSpecificFn       = unsafe extern "C" fn(array_typeinfo: *const Il2CppClass, length: usize) -> *mut Il2CppArray;
pub type Il2CppArrayNewFn               = unsafe extern "C" fn(element_typeinfo: *const Il2CppClass, length: usize) -> *mut Il2CppArray;
pub type Il2CppTypeGetObjectFn          = unsafe extern "C" fn(ty: *const Il2CppType) -> *mut Il2CppReflectionType;
pub type Il2CppClassFromIl2CppTypeFn    = unsafe extern "C" fn(ty: *const Il2CppType) -> *mut Il2CppClass;
pub type Il2CppClassInitFn              = unsafe extern "C" fn(class: *const Il2CppClass);
pub type Il2CppFieldSetValueRawFn       = unsafe extern "C" fn(ty: *const Il2CppType, dest: *mut u8, value: *const u8, deref_pointer: bool);
pub type Il2CppStringNewFn              = unsafe extern "C" fn(c_str: *const u8) -> *mut Il2CppString;

// Filled in by init(), forwarders panic if read before that runs
pub(crate) static REAL_IL2CPP_INIT:                       OnceLock<Il2CppInitFn>                   = OnceLock::new();
pub(crate) static REAL_IL2CPP_ASSEMBLY_GET_IMAGE:         OnceLock<Il2CppAssemblyGetImageFn>       = OnceLock::new();
pub(crate) static REAL_IL2CPP_CLASS_FROM_NAME:            OnceLock<Il2CppClassFromNameFn>          = OnceLock::new();
pub(crate) static REAL_IL2CPP_OBJECT_NEW:                 OnceLock<Il2CppObjectNewFn>              = OnceLock::new();
pub(crate) static REAL_IL2CPP_CLASS_GET_METHOD_FROM_NAME: OnceLock<Il2CppClassGetMethodFromNameFn> = OnceLock::new();
pub(crate) static REAL_IL2CPP_ARRAY_NEW_SPECIFIC:         OnceLock<Il2CppArrayNewSpecificFn>       = OnceLock::new();
pub(crate) static REAL_IL2CPP_ARRAY_NEW:                  OnceLock<Il2CppArrayNewFn>               = OnceLock::new();
pub(crate) static REAL_IL2CPP_TYPE_GET_OBJECT:            OnceLock<Il2CppTypeGetObjectFn>          = OnceLock::new();
pub(crate) static REAL_IL2CPP_CLASS_FROM_IL2CPPTYPE:      OnceLock<Il2CppClassFromIl2CppTypeFn>    = OnceLock::new();
pub(crate) static REAL_IL2CPP_CLASS_INIT:                 OnceLock<Il2CppClassInitFn>              = OnceLock::new();
pub(crate) static REAL_IL2CPP_FIELD_SET_VALUE_RAW:        OnceLock<Il2CppFieldSetValueRawFn>       = OnceLock::new();
pub(crate) static REAL_IL2CPP_STRING_NEW:                 OnceLock<Il2CppStringNewFn>              = OnceLock::new();

// Patterns used to live in unity/unity2, they're here now so there's one owner
const PATTERN_IL2CPP_INIT: &str =
    "fd 7b be a9 f3 0b 00 f9 fd 03 00 91 f3 03 00 aa ?? ?? ?? ?? ?? ?? ?? ?? c0 00 80 52 ?? ?? ?? ?? e0 03 13 aa ?? ?? ?? ?? f3 0b 40 f9 00 00 00 12 fd 7b c2 a8 c0 03 5f d6";
const PATTERN_IL2CPP_ASSEMBLY_GET_IMAGE: &str =
    "ff 03 01 d1 fd 7b 01 a9 fd 43 00 91 f6 57 02 a9 f4 4f 03 a9 f3 03 00 aa ?? ?? ?? ?? ?? ?? ?? ?? 08 21 32 91 f4 03 00 aa ?? ?? ?? ?? df 02 08 eb 80 01 00 54 ?? ?? ?? ??";
const PATTERN_IL2CPP_CLASS_FROM_NAME: &str =
    "ff c3 02 d1 fd 7b 05 a9 fd 43 01 91 fc 6f 06 a9 fa 67 07 a9 f8 5f 08 a9 f6 57 09 a9 f4 4f 0a a9 f8 03 00 aa 16 0f 43 f8 f9 03 02 aa f4 03 01 aa";
const PATTERN_IL2CPP_OBJECT_NEW: &str =
    "ff 43 01 d1 fd 7b 01 a9 fd 43 00 91 f7 13 00 f9 f6 57 03 a9 f4 4f 04 a9 08 c8 44 39 f3 03 00 aa e8 02 10 37";
const PATTERN_IL2CPP_CLASS_GET_METHOD_FROM_NAME: &str =
    "ff 43 01 d1 fd 7b 01 a9 fd 43 00 91 f8 5f 02 a9 f6 57 03 a9 f4 4f 04 a9 08 c8 44 39 f3 03 03 2a f6 03 02 2a f4 03 01 aa f5 03 00 aa";
const PATTERN_IL2CPP_ARRAY_NEW_SPECIFIC: &str =
    "ff 03 01 d1 fd 7b 01 a9 fd 43 00 91 f6 57 02 a9 f4 4f 03 a9 08 c8 44 39 f4 03 01 aa f3 03 00 aa";
const PATTERN_IL2CPP_ARRAY_NEW: &str =
    "fd 7b be a9 f3 0b 00 f9 fd 03 00 91 f3 03 01 aa 21 00 80 52 e2 03 1f 2a ?? ?? ?? ?? e1 03 13 aa f3 0b 40 f9 fd 7b c2 a8";
const PATTERN_IL2CPP_TYPE_GET_OBJECT: &str =
    "ff 03 01 d1 fd 7b 01 a9 fd 43 00 91 f5 13 00 f9 f4 4f 03 a9 ?? ?? ?? ?? ?? ?? ?? ?? a0 0f 00 f9 e0 03 13 aa ff 07 00 f9";
const PATTERN_IL2CPP_CLASS_FROM_IL2CPPTYPE: &str =
    "ff 03 01 d1 fd 7b 01 a9 fd 43 00 91 f6 57 02 a9 f4 4f 03 a9 f3 03 00 aa e0 03 1f aa 68 2a 40 39 08 05 00 51 1f 75 00 71";
const PATTERN_IL2CPP_CLASS_INIT: &str =
    "fd 7b bd a9 f5 0b 00 f9 fd 03 00 91 f4 4f 02 a9 08 c8 44 39 08 03 10 37 ?? ?? ?? ?? ?? ?? ?? ?? f3 03 00 aa b5 0f 00 f9";
const PATTERN_IL2CPP_FIELD_SET_VALUE_RAW: &str =
    "fd 7b bc a9 f7 0b 00 f9 fd 03 00 91 f6 57 02 a9 f4 4f 03 a9 08 08 40 b9 f4 03 02 aa f3 03 01 aa e8 0b f0 37 f5 03 03 2a f6 03 00 aa";
const PATTERN_IL2CPP_STRING_NEW: &str =
    "ff 03 01 d1 fd 7b 02 a9 fd 83 00 91 f4 4f 03 a9 f3 03 00 aa ?? ?? ?? ?? 01 7c 40 92 e8 23 00 91 e0 03 13 aa f4 23 00 91 ?? ?? ?? ?? e8 23 40 39 0b fd 41 d3 e9 0f 40 f9";

/// Scan for every il2cpp function and store the address.
///
/// Call this before any skyline hook is set up, otherwise the pattern
/// for il2cpp_init won't match (it's been hooked at that point).
/// Panics on a miss because handing plugins a null pointer is worse.
pub fn init() {
    let text = unsafe {
        let base = getRegionAddress(Region::Text) as *const u8;
        let size = (getRegionAddress(Region::Rodata) as usize) - (base as usize);
        std::slice::from_raw_parts(base, size)
    };

    // ignoring set errors so calling init twice is harmless
    let _ = REAL_IL2CPP_INIT.set(scan(text, "il2cpp_init", PATTERN_IL2CPP_INIT));
    let _ = REAL_IL2CPP_ASSEMBLY_GET_IMAGE.set(scan(text, "il2cpp_assembly_get_image", PATTERN_IL2CPP_ASSEMBLY_GET_IMAGE));
    let _ = REAL_IL2CPP_CLASS_FROM_NAME.set(scan(text, "il2cpp_class_from_name", PATTERN_IL2CPP_CLASS_FROM_NAME));
    let _ = REAL_IL2CPP_OBJECT_NEW.set(scan(text, "il2cpp_object_new", PATTERN_IL2CPP_OBJECT_NEW));
    let _ = REAL_IL2CPP_CLASS_GET_METHOD_FROM_NAME.set(scan(text, "il2cpp_class_get_method_from_name", PATTERN_IL2CPP_CLASS_GET_METHOD_FROM_NAME));
    let _ = REAL_IL2CPP_ARRAY_NEW_SPECIFIC.set(scan(text, "il2cpp_array_new_specific", PATTERN_IL2CPP_ARRAY_NEW_SPECIFIC));
    let _ = REAL_IL2CPP_ARRAY_NEW.set(scan(text, "il2cpp_array_new", PATTERN_IL2CPP_ARRAY_NEW));
    let _ = REAL_IL2CPP_TYPE_GET_OBJECT.set(scan(text, "il2cpp_type_get_object", PATTERN_IL2CPP_TYPE_GET_OBJECT));
    let _ = REAL_IL2CPP_CLASS_FROM_IL2CPPTYPE.set(scan(text, "il2cpp_class_from_il2cpptype", PATTERN_IL2CPP_CLASS_FROM_IL2CPPTYPE));
    let _ = REAL_IL2CPP_CLASS_INIT.set(scan(text, "il2cpp_class_init", PATTERN_IL2CPP_CLASS_INIT));
    let _ = REAL_IL2CPP_FIELD_SET_VALUE_RAW.set(scan(text, "il2cpp_field_set_value_raw", PATTERN_IL2CPP_FIELD_SET_VALUE_RAW));
    let _ = REAL_IL2CPP_STRING_NEW.set(scan(text, "il2cpp_string_new", PATTERN_IL2CPP_STRING_NEW));

    // Nothing in Cobalt itself uses these, only plugins do. The linker
    // can't see that and drops them unless we touch them somewhere
    forwarders::pin();
}

// F is always a function pointer (8 bytes on aarch64), lazysimd makes
// sure the offset stays inside text
fn scan<F: Copy>(text: &[u8], name: &str, pattern: &str) -> F {
    let offset = lazysimd::get_offset_neon(text, pattern)
        .unwrap_or_else(|| panic!("[cobalt::api::il2cpp] couldn't find {name}, wrong game version?"));
    let addr = text.as_ptr() as usize + offset;
    unsafe { std::mem::transmute_copy(&addr) }
}
