use unity::il2cpp::Il2CppClass;

use super::registry;

#[no_mangle]
pub unsafe extern "C" fn cobapi_register_injected_class(class: *mut Il2CppClass) -> i32 {
    if class.is_null() {
        return 1;
    }
    if registry::insert_raw(class) {
        0
    } else {
        2
    }
}
