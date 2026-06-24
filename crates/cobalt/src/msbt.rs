use engage::app::mess::Mess;
use engage::app::msgfile::{IMsgFileMethods, MsgFile};
use engage::nintendo::message_studio::lib::binlibmsfilebase::IBinLibmsFileBaseMethods;
use engage::nintendo::message_studio::lib::binmsgfile::IBinMsgFileMethods;
use engage::system::collections::generic::dictionary_2::IDictionary_2Methods as IDictionaryMethods;
use unity::prelude::*;

use crate::api::events::{publish_system_event, SystemEvent};

pub fn add_virtual_msbt() {
    let msg_file = MsgFile::new();

    let messages = astra_formats::pack_astra_script(&format!(
        "[MID_SYS_Mode_Phoenix]\n{}\n\n[MID_H_Mode_Phoenix]\n{}\n\n",
        localize::mess::get("phoenix_mode_name"),
        localize::mess::get("phoenix_mode_help")
    ))
    .unwrap();

    let mut message_map = astra_formats::MessageMap {
        num_buckets: 2,
        messages,
    };
    let msbt = message_map.serialize().unwrap();
    let bytes = Array::<u8>::from_slice(&msbt).expect("msbt bytes array");

    msg_file.load(bytes);
    msg_file.set_reference(1);

    let mess_file_dict = Mess::s_mess_file_dictionary();
    let mess_data_dict = Mess::s_mess_data_dictionary();
    let path_dict = Mess::s_path_dictionary();

    let dummy_path = Il2CppString::new("dummy");
    mess_file_dict.add(dummy_path, msg_file);

    let text_num = msg_file.get_text_num();
    for i in 0..text_num {
        let label = msg_file.get_label(i);
        let text = msg_file.get_text_2(i);
        mess_data_dict.add(label, text);
        path_dict.add(label, dummy_path);
    }
}

#[unity::hook("App", "Language", "ReflectSetting")]
pub fn language_reflectchange(method_info: OptionalMethod) {
    call_original!(method_info);
    localize::mess::initialize(Mess::get_language_directory_name().to_string()).unwrap();
    add_virtual_msbt();
    publish_system_event(SystemEvent::LanguageChanged);
}
