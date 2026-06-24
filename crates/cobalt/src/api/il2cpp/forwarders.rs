//! The actual exported il2cpp_* functions plugins call into.
//!
//! Bodies just hand off to the scanned address for now. Hook code will
//! live here when we have any, that's the whole reason this layer exists

use super::*;

#[no_mangle]
pub unsafe extern "C" fn il2cpp_init(domain_name: *const i8) -> i32 {
    (REAL_IL2CPP_INIT.get().expect("il2cpp_init called before cobalt::api::il2cpp::init"))(domain_name)
}

#[no_mangle]
pub unsafe extern "C" fn il2cpp_assembly_get_image(c_str: *const u8) -> *const Il2CppImage {
    (REAL_IL2CPP_ASSEMBLY_GET_IMAGE.get().expect("il2cpp_assembly_get_image called before cobalt::api::il2cpp::init"))(c_str)
}

#[no_mangle]
pub unsafe extern "C" fn il2cpp_class_from_name(
    image: *const Il2CppImage,
    namespace: *const u8,
    name: *const u8,
) -> *mut Il2CppClass {
    // injected classes win, same registry-first order as the game-side
    // mono_class_from_assembly_namespace_name hook
    if let Some(class) = crate::injection::registry::lookup_by_name_raw(namespace, name) {
        return (class as *mut unity::il2cpp::Il2CppClass).cast();
    }
    (REAL_IL2CPP_CLASS_FROM_NAME.get().expect("il2cpp_class_from_name called before cobalt::api::il2cpp::init"))(image, namespace, name)
}

#[no_mangle]
pub unsafe extern "C" fn il2cpp_object_new(klass: *const Il2CppClass) -> *mut Il2CppObject {
    (REAL_IL2CPP_OBJECT_NEW.get().expect("il2cpp_object_new called before cobalt::api::il2cpp::init"))(klass)
}

#[no_mangle]
pub unsafe extern "C" fn il2cpp_class_get_method_from_name(
    class: *const Il2CppClass,
    method_name: *const u8,
    args_count: usize,
    flags: u32,
) -> *mut MethodInfo {
    (REAL_IL2CPP_CLASS_GET_METHOD_FROM_NAME.get().expect("il2cpp_class_get_method_from_name called before cobalt::api::il2cpp::init"))(class, method_name, args_count, flags)
}

#[no_mangle]
pub unsafe extern "C" fn il2cpp_array_new_specific(
    array_typeinfo: *const Il2CppClass,
    length: usize,
) -> *mut Il2CppArray {
    (REAL_IL2CPP_ARRAY_NEW_SPECIFIC.get().expect("il2cpp_array_new_specific called before cobalt::api::il2cpp::init"))(array_typeinfo, length)
}

#[no_mangle]
pub unsafe extern "C" fn il2cpp_array_new(
    element_typeinfo: *const Il2CppClass,
    length: usize,
) -> *mut Il2CppArray {
    (REAL_IL2CPP_ARRAY_NEW.get().expect("il2cpp_array_new called before cobalt::api::il2cpp::init"))(element_typeinfo, length)
}

#[no_mangle]
pub unsafe extern "C" fn il2cpp_type_get_object(ty: *const Il2CppType) -> *mut Il2CppReflectionType {
    (REAL_IL2CPP_TYPE_GET_OBJECT.get().expect("il2cpp_type_get_object called before cobalt::api::il2cpp::init"))(ty)
}

#[no_mangle]
pub unsafe extern "C" fn il2cpp_class_from_il2cpptype(ty: *const Il2CppType) -> *mut Il2CppClass {
    (REAL_IL2CPP_CLASS_FROM_IL2CPPTYPE.get().expect("il2cpp_class_from_il2cpptype called before cobalt::api::il2cpp::init"))(ty)
}

#[no_mangle]
pub unsafe extern "C" fn il2cpp_class_init(class: *const Il2CppClass) {
    (REAL_IL2CPP_CLASS_INIT.get().expect("il2cpp_class_init called before cobalt::api::il2cpp::init"))(class)
}

#[no_mangle]
pub unsafe extern "C" fn il2cpp_field_set_value_raw(
    ty: *const Il2CppType,
    dest: *mut u8,
    value: *const u8,
    deref_pointer: bool,
) {
    (REAL_IL2CPP_FIELD_SET_VALUE_RAW.get().expect("il2cpp_field_set_value_raw called before cobalt::api::il2cpp::init"))(ty, dest, value, deref_pointer)
}

#[no_mangle]
pub unsafe extern "C" fn il2cpp_string_new(c_str: *const u8) -> *mut Il2CppString {
    (REAL_IL2CPP_STRING_NEW.get().expect("il2cpp_string_new called before cobalt::api::il2cpp::init"))(c_str)
}

/// Touch every forwarder so the linker keeps them in the build.
///
/// Cobalt never calls these itself, only plugins do, and the linker can't
/// see plugin calls so it'll throw them out unless we point at them here
pub(super) fn pin() {
    let _kept: &[*const ()] = &[
        il2cpp_init as *const (),
        il2cpp_assembly_get_image as *const (),
        il2cpp_class_from_name as *const (),
        il2cpp_object_new as *const (),
        il2cpp_class_get_method_from_name as *const (),
        il2cpp_array_new_specific as *const (),
        il2cpp_array_new as *const (),
        il2cpp_type_get_object as *const (),
        il2cpp_class_from_il2cpptype as *const (),
        il2cpp_class_init as *const (),
        il2cpp_field_set_value_raw as *const (),
        il2cpp_string_new as *const (),
    ];
    std::hint::black_box(_kept);
}
