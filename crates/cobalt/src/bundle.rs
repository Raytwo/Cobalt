use unity::prelude::*;

use engage::app::textassetbundle::{ITextAssetBundleMethods, TextAssetBundle};
use engage::unity_engine::assetbundle::AssetBundle;
use engage::unity_engine::assetbundlecreaterequest::AssetBundleCreateRequest;

use engage::AssetBundleExt;

use camino::Utf8PathBuf;

#[unity::hook("UnityEngine", "AssetBundle", "LoadFromFileAsync")]
pub fn load_from_file_async_hook(path: Il2CppString, method_info: OptionalMethod) -> AssetBundleCreateRequest {
    let orig_path = Utf8PathBuf::from(path.to_rust_string());

    match mods::read(orig_path.strip_prefix("rom:/").unwrap()) {
        Ok(content) => {
            let array = Array::<u8>::from_slice(&content).unwrap();
            let r = AssetBundle::load_from_memory_async_internal(array, 0u32);
            r
        }
        Err(_) => {
            let r = call_original!(path, method_info);
            r
        }
    }
}

#[unity::hook("App", "TextAssetBundle", ".ctor")]
pub fn textassetbundle_ctor_hook(this: TextAssetBundle, path: Il2CppString, method_info: OptionalMethod) {
    let str_path = path.to_rust_string();

    if str_path.starts_with("Scripts") {
        let filepath = format!("patches/{}", str_path.to_lowercase());

        if let Ok(content) = mods::read(&filepath) {
            println!("Replacing Lua script: {}", str_path);

            let script = std::str::from_utf8(&content)
                .unwrap_or_else(|_| panic!("Could not read Lua patch at location '{}' as UTF8", filepath))
                .as_bytes()
                .to_vec();

            let array = Array::<u8>::from_slice(&script).unwrap();

            this.set_bytes(array);
            return;
        }
    }

    call_original!(this, path, method_info);

    let bytes = this.get_bytes();
    if bytes.is_null() {
        return;
    }

    // Only care about MSBT files for now
    if !str_path.starts_with("Message") {
        return;
    }

    let regional_path = Utf8PathBuf::from("patches/msbt/").join(str_path.to_lowercase()).with_extension("txt");

    let message_path = if mods::contains(&regional_path) {
        regional_path
    } else if !(str_path.contains("Puppet") || str_path.contains("Sound")) {
        Utf8PathBuf::from("patches/msbt/message/").join(regional_path.file_name().unwrap())
    } else {
        return;
    };

    let mut original_msbt = astra_formats::MessageMap::from_slice(bytes.as_slice()).unwrap_or_else(|_| {
        panic!(
            "Astra-formats could not parse the game's {} MSBT.\n\nMake sure you do not have a malformed leftover bundled MSBT in your romfs or in a Cobalt mod.\n\nIf you don't, please open an issue on Cobalt's repository with the following message and the name of the file",
            str_path
        )
    });

    mods::read_layers(&message_path)
        .iter()
        .flatten()
        .map(|content| {
            println!("Patching using text MSBT: {}", message_path);
            let script = std::str::from_utf8(content)
                .unwrap_or_else(|_| panic!("Could not parse txt MSBT patch at location '{}' as UTF8", message_path));
            astra_formats::pack_astra_script(script)
                .unwrap_or_else(|_| panic!("should have parsed the txt MSBT patch at location '{}'", message_path))
        })
        .for_each(|messages| {
            for (label, text) in messages {
                match original_msbt.messages.get_mut(&label) {
                    Some(entry) => {
                        if *entry != text {
                            *entry = text;
                        }
                    }
                    None => {
                        original_msbt.messages.insert(label, text);
                    }
                }
            }
        });

    let serialized_msbt = original_msbt.serialize().unwrap();
    let array = Array::<u8>::from_slice(&serialized_msbt).unwrap();

    this.set_bytes(array);
}
