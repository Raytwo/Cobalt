#![allow(unused)]

extern "C" {
    pub fn sky_GetModuleInfo(index: ModuleIndex) -> &'static ModuleInfo;
    pub fn sky_Memcpy(dest: *mut u8, src: *const u8, size: usize) -> u32;

    #[link_name = "_ZN3exl4util10mem_layout6s_HeapE"]
    pub static s_Heap: Range;
}

#[repr(u8)]
pub enum Region {
    Text,
    Rodata,
    Data,
    Bss,
    Heap,
}

#[repr(u8)]
pub enum ModuleIndex {
    Rtld,
    Main,
}

#[repr(C)]
pub struct Range {
    pub start: *const u8,
    pub size: usize,
}

#[repr(C)]
pub struct ModuleInfo {
    pub total: Range,
    pub text: Range,
    pub rodata: Range,
    pub data: Range,
}
