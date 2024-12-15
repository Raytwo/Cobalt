// Extern block to link into exlaunch's initialization requirements
extern "C" {
    #[link_name = "_ZN3exl4hook4nx6410InitializeEv"]
    fn exl_hook_initialize();

    fn exl_init();
    fn __init_array();
}

#[no_mangle]
unsafe extern "C" fn __custom_init() {
    exl_init();
    __init_array();
    exl_hook_initialize();
    crate::main();
    // exl_main(0 as _, 0 as _);
}

#[no_mangle]
unsafe extern "C" fn __custom_fini() {}
