use std::collections::HashMap;
use std::ffi::c_void;
use std::slice;
use std::str;
use std::sync::LazyLock;
use std::sync::RwLock;

use cobapi::VTableHeader;
use cobapi::VTABLE_ABI_VERSION;

struct Entry {
    this: *mut c_void,
    vtable: *const VTableHeader,
}

unsafe impl Send for Entry {}
unsafe impl Sync for Entry {}

static REGISTRY: LazyLock<RwLock<HashMap<String, Entry>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

const RC_OK: i32 = 0;
const RC_ALREADY_REGISTERED: i32 = 1;
const RC_ABI_MISMATCH: i32 = 2;
const RC_NOT_FOUND: i32 = 3;

#[no_mangle]
pub unsafe extern "C" fn cobapi_register_service(
    id_ptr: *const u8,
    id_len: usize,
    this: *mut c_void,
    vtable: *const VTableHeader,
) -> i32 {
    let Some(id) = id_from_raw(id_ptr, id_len) else {
        return RC_NOT_FOUND;
    };

    if vtable.is_null() {
        return RC_ABI_MISMATCH;
    }

    let header = unsafe { &*vtable };

    if header.abi_version != VTABLE_ABI_VERSION {
        return RC_ABI_MISMATCH;
    }

    let mut map = REGISTRY.write().unwrap();

    if map.contains_key(id) {
        return RC_ALREADY_REGISTERED;
    }

    map.insert(id.to_owned(), Entry { this, vtable });

    RC_OK
}

#[no_mangle]
pub unsafe extern "C" fn cobapi_lookup_service(
    id_ptr: *const u8,
    id_len: usize,
    out_this: *mut *mut c_void,
    out_vtable: *mut *const VTableHeader,
) -> i32 {
    let Some(id) = id_from_raw(id_ptr, id_len) else {
        return RC_NOT_FOUND;
    };

    let map = REGISTRY.read().unwrap();

    let Some(entry) = map.get(id) else {
        return RC_NOT_FOUND;
    };

    unsafe {
        *out_this = entry.this;
        *out_vtable = entry.vtable;
    }

    RC_OK
}

#[no_mangle]
pub unsafe extern "C" fn cobapi_unregister_service(id_ptr: *const u8, id_len: usize) {
    let Some(id) = id_from_raw(id_ptr, id_len) else {
        return;
    };

    REGISTRY.write().unwrap().remove(id);
}

unsafe fn id_from_raw<'a>(ptr: *const u8, len: usize) -> Option<&'a str> {
    if ptr.is_null() || len == 0 {
        return None;
    }

    let bytes = unsafe { slice::from_raw_parts(ptr, len) };

    str::from_utf8(bytes).ok()
}
