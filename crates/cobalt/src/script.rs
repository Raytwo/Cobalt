use unity::prelude::*;
use engage::{eventsequence::EventSequence, proc::ProcInst, script::*};

use crate::api::lua::ScriptCobalt;

#[unity::hook("App", "ScriptSystem", "Log")]
pub fn scriptsystem_log(args: &Il2CppArray<DynValue>, _method_info: OptionalMethod) {

    if let Some(string) = args.try_get_string(0) {
        println!("ScriptSystem::Log: {}", string.to_string());
    }
}

#[unity::hook("App", "HubSequence", "MapOpeningEvent")]
pub fn hubsequence_mapopeningevent_hook(this: &ProcInst, method_info: OptionalMethod) {
    // println!("App.HubSequence::MapOpeningEvent");
    
    // Only run custom lua files in the Hub until Dragon's Gate is ready
    if engage::gameuserdata::GameUserData::get_sequence() == 4 {
        println!("Reading custom lua");
        unsafe {
            let manager = mods::manager::Manager::get();
            let root_dir = manager.get_directory("").unwrap();
    
            manager.get_files_in_directory(root_dir)
                .unwrap()
                .iter()
                .filter(|relative| relative.extension() == Some("lua"))
                .for_each(|lua_path| {
                    println!("lua path: {}", lua_path);
                    // TODO: Rewrite this to not require the TextAssetBundle::.ctor hook
                    let script = manager.get_file(&lua_path).unwrap();
                    // let script = std::fs::read(lua_path.first().unwrap()).unwrap();
                    let array = Il2CppArray::from_slice(script).unwrap();
                    let instance = EventScript::get_instance();
                    let stream = MemoryStream::instantiate().unwrap();
                    system_io_memorystream_ctor(stream, array, None);
                    moonsharp_interpreter_script_dostream(instance, stream, 0 as _, 0 as _, None);
                    let func =
                        moonsharp_interpreter_table_get(instance.global_table, lua_path.file_stem().unwrap().into(), None);
                    EventSequence::try_create_bind(this, func, None, None, None);
                })
        }
    }

    call_original!(this, method_info);
}

#[unity::from_offset("App", "EventScript", "Load")]
pub fn eventscript_load(path: &Il2CppString, method_info: OptionalMethod);

#[unity::from_offset("App", "EventScript", "Unload")]
pub fn eventscript_unload(method_info: OptionalMethod);

#[unity::hook("App", "EventScript", "LoadImpl")]
pub fn eventscript_loadimpl_hook(this: &'static EventScript, path: &Il2CppString, method_info: OptionalMethod) {
    call_original!(this, path, method_info);

    ScriptCobalt::register(this);
}
