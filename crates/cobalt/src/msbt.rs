use unity::prelude::*;

use engage::mess::*;

use crate::api::events::{publish_system_event, SystemEvent};

pub fn add_virtual_msbt() {
    let mut msg_file = MsgFile::instantiate().unwrap();
    unsafe { msgfile_ctor(&msg_file, None) };

    let messages = astra_formats::pack_astra_script(
        &format!("[MID_SYS_Mode_Phoenix]\n{}\n\n[MID_H_Mode_Phoenix]\n{}\n\n", localize::mess::get("phoenix_mode_name"), localize::mess::get("phoenix_mode_help"))
    ).unwrap();

    let mut message_map = astra_formats::MessageMap {
        num_buckets: 2,
        messages
    };

    let msbt = message_map.serialize().unwrap();

    let array = Il2CppArray::from_slice(msbt).unwrap();

    unsafe { msbt_load(&mut msg_file, array, None) };

    let mess = Mess::class();
    let static_fields = mess.get_static_fields::<MessStaticFields>();

    msg_file.reference_count = 1;

    static_fields.mess_file_dictionary.add("dummy".into(), msg_file);

    let text_num = unsafe { msbt_get_text_num(&msg_file, None) };

    // Process each entry. We could use astra_format's msbt support eventually
    for i in 0..text_num as usize {
        let label = unsafe { msbt_get_label(&msg_file, i, None) };
        let text = unsafe { msbt_get_text(&msg_file, i, None) };

        static_fields.mess_data_dictionary.add(label, text);
        static_fields.path_dictionary.add(label, "dummy".into());
    }
}

#[skyline::hook(offset = 0x25d2750)]
pub fn mess_initialize(method_info: OptionalMethod) {
    call_original!(method_info);
}

#[unity::hook("App", "Language", "ReflectSetting")]
pub fn language_reflectchange(method_info: OptionalMethod) {
    call_original!(method_info);
    // The system savedata finished loading, so we can access the user's language.
    localize::mess::initialize(Mess::get_language_directory_name().to_string()).unwrap();
    add_virtual_msbt();
    // load_custom_msbt();
    publish_system_event(SystemEvent::LanguageChanged);
}
