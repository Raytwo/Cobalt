use engage::app::eventscript::EventScript;
use engage::app::eventsequence::{EventSequence, IEventSequenceMethods};
use engage::app::gameuserdata::{GameUserData, GameUserData_Sequences, IGameUserDataMethods};
use engage::app::procinst::{IProcInstMethods, ProcInst};
use engage::app::scriptutil::ScriptUtil;
use engage::app::singletonclass_1::ISingletonClass_1Methods;
use engage::moon_sharp::interpreter::dynvalue::DynValue;
use engage::moon_sharp::interpreter::script::{IScript, IScriptMethods, Script};
use engage::moon_sharp::interpreter::table::{ITableMethods, Table};
use engage::system::io::memorystream::MemoryStream;
use unity::prelude::*;
use unity::FromIlInstance;

use engage::DynValueArgs;

use crate::api::lua::ScriptCobalt;

#[unity::hook("App", "ScriptSystem", "Log")]
pub fn scriptsystem_log(args: Array<DynValue>, _method_info: OptionalMethod) {
    let first = args.try_get_string(0);
    if !first.is_null() {
        println!("ScriptSystem::Log: {}", first);
    }
}

#[unity::hook("App", "EventSequence", "MapOpening")]
pub fn eventsequence_mapopening_hook(parent: ProcInst, method_info: OptionalMethod) {
    call_original!(parent, method_info);

    let seq = GameUserData::get_instance().get_sequence();
    if seq != GameUserData_Sequences::hub() {
        return;
    }

    println!("Reading custom lua");

    let child = parent.get_child();
    if child.is_null() {
        return;
    }
    let child_sequence = <EventSequence as FromIlInstance>::from_il_instance(child.as_instance());
    let instance = EventScript::get_instance();

    mods::list("")
        .filter(|relative| relative.extension() == Some("lua"))
        .for_each(|lua_path| {
            println!("lua path: {}", lua_path);
            // TODO: rewrite this so it doesn't need the TextAssetBundle::.ctor hook
            let Ok(script) = mods::read(lua_path) else {
                return;
            };
            let Some(array) = Array::<u8>::from_slice(&script) else {
                return;
            };

            let stream = MemoryStream::new_3(array);

            let script_view = <Script as FromIlInstance>::from_il_instance(
                <EventScript as Into<IlInstance>>::into(instance),
            );
            let null_string = Il2CppString::from_il_instance(IlInstance::null());
            let null_table = <Table as FromIlInstance>::from_il_instance(IlInstance::null());
            let stream_as_stream: engage::system::io::stream::Stream = stream.into();
            script_view.do_stream(stream_as_stream, null_table, null_string);

            let func_name: Il2CppString = lua_path.file_stem().unwrap().into();
            let func = instance.m_global_table().get(func_name);
            if func.is_null() {
                return;
            }
            let null_args: Array<DynValue> = Array::null();
            child_sequence.add_coroutine(func, null_args);
            let _ = script_view;
            let _ = ScriptUtil::class();
        });
}

#[unity::hook("App", "EventScript", "LoadImpl")]
pub fn eventscript_loadimpl_hook(this: EventScript, path: Il2CppString, method_info: OptionalMethod) {
    call_original!(this, path, method_info);
    ScriptCobalt::register(this);
}
