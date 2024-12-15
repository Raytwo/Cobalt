use unity::prelude::*;
use unity::engine::AssetBundle;

use engage::bundle::TextAssetBundle;

use camino::Utf8PathBuf;

#[unity::from_offset("UnityEngine", "AssetBundle", "LoadFromMemoryAsync_Internal(System.Byte[],System.UInt32)")]
pub fn assetbundle_loadfrommemoryasync_internal(file: &'static mut Il2CppArray<u8>, idk: u32, method_info: OptionalMethod) -> *const u8;

#[skyline::hook(offset = 0x3f41ff0)]
pub fn irawbundle_load_hook(path: &Il2CppString, method_info: OptionalMethod) -> *const u8 {
    let orig_path = &mut Utf8PathBuf::from(&path.to_string());

    match mods::manager::Manager::get().get_file(orig_path.strip_prefix("rom:/").unwrap()) {
        Ok(content) => {
            let array = Il2CppArray::from_slice(content).unwrap();

            AssetBundle::load_from_memory_async_internal(array, 0)
        },
        Err(_) => call_original!(path, method_info),
    }
}

#[unity::hook("App", "TextAssetBundle", ".ctor")]
pub fn textassetbundle_ctor_hook(this: &mut TextAssetBundle, path: &Il2CppString, method_info: OptionalMethod) {
    let str_path = path.to_string();

    // println!("TextAssetBundle::.ctor path: {}", str_path);

    if str_path.starts_with("Scripts") {
        let filepath = format!("patches/{}", str_path.to_lowercase());

        if let Ok(mut content) = mods::manager::Manager::get().get_file(&filepath) {
            // lua handling
            println!("Replacing Lua script: {}", str_path);

            let script = std::str::from_utf8(&mut content).unwrap_or_else(|_| {
                panic!("Could not read Lua patch at location '{}' as UTF8", filepath);
            }).as_bytes().to_vec();
            
            let array = Il2CppArray::from_slice(script).unwrap();

            this.bytes = Some(array);

            return;
        }
    }

    call_original!(this, path, method_info);

    // Check if the file was properly read
    if let Some(bytes) = &this.bytes {
        // Only care about MSBT files for now
        if str_path.starts_with("Message") {
            let manager = mods::manager::Manager::get();

            let regional_path = Utf8PathBuf::from("patches/msbt/").join(str_path.to_lowercase()).with_extension("txt");

            let message_path = if manager.exists(&regional_path) {
                regional_path
            } else if !(str_path.contains("Puppet") || str_path.contains("Sound")) {
                Utf8PathBuf::from("patches/msbt/message/").join(regional_path.file_name().unwrap())
            } else {
                // We don't have a file to patch and the game already loaded the original copy, so abort here.
                return;
            };

            let mut original_msbt = astra_formats::MessageMap::from_slice(bytes).unwrap_or_else(|_| {
                panic!("Astra-formats could not parse the game's {} MSBT.\n\nMake sure you do not have a malformed leftover bundled MSBT in your romfs or in a Cobalt mod.\n\nIf you don't, please open an issue on Cobalt's repository with the following message and the name of the file", str_path)
            });

            manager
                .get_files(&message_path)
                .iter()
                .flatten()
                .map(|content| {
                    // MSBT handling
                    println!("Patching using text MSBT: {}", message_path);
                    let script = std::str::from_utf8(content).unwrap_or_else(|_| {
                        panic!(
                            "Could not parse txt MSBT patch at location '{}' as UTF8",
                            message_path
                        );
                    });
                    let patch = astra_formats::pack_astra_script(script).expect(&format!(
                        "should have parsed the txt MSBT patch at location '{}'",
                        message_path
                    ));
                    patch
                })
                .for_each(|messages| {
                    for (label, text) in messages {
                        match original_msbt.messages.get_mut(&label) {
                            Some(entry) => {
                                if *entry != text {
                                    *entry = text;
                                }
                            },
                            None => {
                                original_msbt.messages.insert(label, text);
                            },
                        }
                    }
                });
            

            let serialized_msbt = original_msbt.serialize().unwrap();
            let array = Il2CppArray::from_slice(serialized_msbt).unwrap();

            this.bytes = Some(array);
        }
    }
}