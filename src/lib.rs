#![feature(if_let_guard, int_roundings, ptr_sub_ptr)]
#![feature(new_zeroed_alloc)]

use std::sync::atomic::{
    AtomicBool,
    Ordering::{Relaxed, SeqCst},
};

use camino::Utf8PathBuf;
use cobalt::vibrationevents::initialize_vibration_data;
use log::Log;
use skyline::{nn, patching::Patch};
use unity::prelude::*;

use crate::utils::env::is_emulator;

mod api;
mod bootstrap;
mod logger;
mod old_api;
mod utils;

static SOCKET_INIT: AtomicBool = AtomicBool::new(false);

#[skyline::hook(replace = nn::socket::Initialize)]
pub fn socket_initialize_hook(pool: *mut u8, pool_size: usize, alloc_pool_size: usize, concur_limit: i32) -> i32 {
    if !SOCKET_INIT.load(Relaxed) {
        println!("[ozone] nn::socket::Initialize called for the first time");
        SOCKET_INIT.store(true, SeqCst);
        call_original!(pool, pool_size, alloc_pool_size, concur_limit)
    } else {
        // Pretend the operation was successful
        println!("[ozone] nn::socket::Initialize dummied out");
        0
    }
}

#[skyline::hook(replace = nn::socket::Initialize_Config)]
pub fn socket_initialize_config_hook(pool: *mut u8) -> i32 {
    if !SOCKET_INIT.load(Relaxed) {
        println!("[ozone] nn::socket::Initialize (Config) called for the first time");
        SOCKET_INIT.store(true, SeqCst);
        call_original!(pool)
    } else {
        // Pretend the operation was successful
        println!("[ozone] nn::socket::Initialize (Config) dummied out");

        0
    }
}

#[skyline::hook(replace = skyline::nn::fs::MountRom)]
pub fn mount_rom_hook(name: *const char, buffer: *const u8, buf_size: usize) -> i32 {
    let _ = unsafe { nn::fs::MountSdCardForDebug(skyline::c_str("sd\0")) };

    println!("[ozone] SD card mounted");

    call_original!(name, buffer, buf_size)
}

#[skyline::hook(offset = il2cpp::il2cpp_init_scan())]
pub fn il2cpp_init_hook(domain_name: *const i8) -> i32 {
    let res = call_original!(domain_name);

    // The il2cpp metadata system is loaded by now, so you can do whatever.
    // This'd be a fine place to call the entry point for plugins.

    // Make sure the paths exist before doing anything
    utils::paths::ensure_paths_exist().expect("Paths should exist on the SD");

    skyline::install_hooks!(
        cobalt::get_patch_name_hook,
        cobalt::catalog::from_json_hook,
        cobalt::config::configmenu_createbind_hook,
        cobalt::bundle::irawbundle_load_hook,
        cobalt::bundle::textassetbundle_ctor_hook,
        cobalt::sortie::sortietopmenushopsubmenu_createbind_hook,
        cobalt::sequences::mainmenu::topmenu::menu::createmenubind_hook,
        cobalt::sequences::mainmenu::mainmenusequence_getdesc_hook,
        cobalt::script::scriptsystem_log,
        cobalt::script::hubsequence_mapopeningevent_hook,
        cobalt::script::eventscript_loadimpl_hook,
        cobalt::msbt::language_reflectchange,
        cobalt::vibrations::combat::hits::combat_damagesound_play,
        cobalt::vibrations::map::map::app_game_sound_hit,
        cobalt::sortie::shopmenuitem_getname_hook,
        cobalt::sortie::godroomunitselectmenu_createbind_hook,
        cobalt::vibrations::combat::projectiles::physical::character_sound_shoot,
        // cobalt::combatvibration::pick_unit_tick, // no vibrating engage for now
        // cobalt::combatvibration::app_map_unit_command_menu_tick, // no vibrating engage for now
        cobalt::ringvibration::clean_ring,
        cobalt::ringvibration::play_rub_effect,
        cobalt::ringvibration::cleaning_start_event,
        cobalt::save::savedatamenu_setupbymenuitem,
        cobalt::combatui::damage_popup,
        cobalt::vibrations::combat::projectiles::magical::cmd_shoot,
        cobalt::combatui::fade_in_hud,
        cobalt::support::get_for_demo_hook,
        cobalt::vibrations::development::reloadhooks::sortie_top_menu_create_bind,
        cobalt::vibrations::development::reloadhooks::map_system_menu_create_bind,
        cobalt::vibrations::sound_events::sound_event_handlers::app_sound_manager_post_event,
        cobalt::vibrations::sound_events::sound_event_handlers::app_sound_manager_post_event_2,
        cobalt::vibrations::hooks::rewind_sequence_start,
        cobalt::vibrations::hooks::rewind_sequence_cancel_rewind,
        cobalt::vibrations::hooks::rewind_sequence_execute_rewind,
        cobalt::vibrations::queue_handlers::field_bgm_manager_tick,
        cobalt::vibrations::queue_handlers::combat_skip_skip,
        cobalt::vibrations::combat::projectiles::rods::cmd_sound,
        cobalt::vibrations::combat::criticals::my_start,
        cobalt::sprite::icon_destroy,
        cobalt::sprite::get_skill_icon,
        cobalt::sprite::get_item_icon_string,
        cobalt::sprite::get_item_icon_itemdata,
        cobalt::sprite::trygetuniticonindex,
        cobalt::sprite::uniticon_tryset_hook,
        cobalt::sprite::facethumb_get_unit,
        cobalt::sprite::facethumb_get_god,
        cobalt::sprite::facethumb_get_ring,
        cobalt::sprite::bondsringfacepicture_get,
        cobalt::sprite::godfacepicture_getsprite,
        cobalt::sprite::gameicon_trygetgodring_unit,
        cobalt::sprite::gameicon_trygetgodring_god,
        cobalt::sprite::gameicon_trygetgodsymbol,
        cobalt::sprite::godcolorrefineemblem_getcolor,
        cobalt::sprite::mapuigauge_getspritebyname,
        cobalt::procinst_jump,
        cobalt::graphics::render_scale::set_render_scale,
        cobalt::graphics::render_scale::set_render_scale_internal,
        cobalt::graphics::lod::lod_hook,
        // cobalt::class_from_name,
        cobalt::audio::gamesound::load_default_sound_banks,
        cobalt::audio::wwise::wwise_file_open_hook,
        cobalt::audio::gamesound::gamesound_personvoice,
        cobalt::audio::unitinfo_reservecharavoice,
        cobalt::audio::gamesound::gamesound_ringcleaningvoice,
        cobalt::audio::soundmanager::soundmanager_postevent_with_temporarygameobject,
        cobalt::audio::soundplay::soundplay_posteventcallback,
        cobalt::audio::gamesound::gamesound_unitpickvoice,
        cobalt::audio::filehandle_loadasync,
        cobalt::audio::soundmanager::soundmanager_iseventplaying,
        cobalt::audio::gamesound::gamesound_loadsystemvoice,
        cobalt::audio::gamesound::gamesound_unloadsystemvoice,
        cobalt::audio::gamesound::gamesound_setenumparam_gameobject,
        cobalt::goddata_getengagezoneprefabpath,
        cobalt::save::gamesavedata_procread_deserialize,
        cobalt::sprite::gmapinfocontent_setmapinfo_hook,
    );

    // XML Patching
    skyline::install_hooks!(
        gamedata::structdata_import,
        gamedata::structdataarray_import,
        gamedata::database_completed_hook,
        cobalt::support::reliancedata_trygetexp,
    );

    // load up vibration data
    initialize_vibration_data();

    // Load plugins found on the SD
    let manager = mods::manager::Manager::get();
    let root_dir = manager.get_directory(Utf8PathBuf::from("")).unwrap();

    let files = manager.get_files_in_directory(root_dir).unwrap();

    files.iter().filter(|entry| entry.extension() == Some("ips")).for_each(|entry| {
        println!("IPS entry: {}", entry);
        let file = manager.get_file(entry).unwrap();
        let patch = ips::Patch::parse(&file).unwrap();

        for hunk in patch.hunks() {
            Patch::in_text(hunk.offset() - 0x100).bytes(hunk.payload()).unwrap();
        }
    });

    println!("Finished applying all patches");

    let nros = files.iter().filter(|entry| entry.extension() == Some("nro")).map(|entry| {
        println!("NRO entry: {}", entry);
        loader::NroFile::from_slice(entry.file_stem().unwrap(), manager.get_file(entry).unwrap()).unwrap()
    });

    let loader_results = loader::mount_plugins(nros);

    match loader_results {
        Ok(info) => unsafe {
            for module in info.modules.iter() {
                match module.as_ref() {
                    Ok(plugin) => {
                        let mut symbol = 0usize;
                        nn::ro::LookupModuleSymbol(&mut symbol, plugin, b"main\0".as_ptr());
                        let nul = plugin
                            .Name
                            .iter()
                            .enumerate()
                            .find(|(_, byte)| **byte == 0)
                            .map(|(count, _)| count)
                            .unwrap();

                        let name = std::str::from_utf8_unchecked(&plugin.Name[0..nul]);

                        if symbol == 0 {
                            println!("[ozone] Plugin {} does not have a main function.", name);
                        } else {
                            let main_func: extern "C" fn() = std::mem::transmute(symbol);
                            println!("[ozone] Calling function 'main' in plugin {}", name);
                            main_func();
                            println!("[ozone] Function 'main' has finished in plugin {}", name);
                        }
                    },
                    Err(e) => println!("[ozone] Error mounting module: {}", e),
                }
            }
        },
        Err(e) => println!("Loader failed: {}", e),
    }

    println!("Finished loading all plugins");

    cobalt::api::events::cobapi_register_system_event_listener(cobalt::sequences::titleloop::grand_opening_skip);
    cobalt::api::events::cobapi_register_system_event_listener(cobalt::updater::catalog_mount_update_check);

    res
}

#[skyline::main]
pub fn main() {
    // Panic handler for Rust panics
    std::panic::set_hook(Box::new(|info| {
        let location = info.location().unwrap();

        let msg = match info.payload().downcast_ref::<&'static str>() {
            Some(s) => *s,
            None => {
                match info.payload().downcast_ref::<String>() {
                    Some(s) => &s[..],
                    None => "Box<Any>",
                }
            },
        };

        println!("Cobalt v{}\nLocation: {}\n\n{}", cobalt::utils::env::get_cobalt_version(), location, msg);

        let err_msg = format!(
            "Cobalt v{}\nLocation: {}\n\n{}\0",
            cobalt::utils::env::get_cobalt_version(),
            location,
            msg
        );

        skyline::error::show_error(69, "Cobalt has panicked! Press 'Details' for more information.\n\0", err_msg.as_str());
    }));

    let current_ver = utils::env::get_game_version();

    if utils::env::get_game_version() != semver::Version::new(2, 0, 0) {
        skyline::error::show_error(69,
            &format!("Cobalt is only compatible with Fire Emblem Engage version 2.0.0, but you are running version {}.", current_ver),
            &format!("Cobalt is only compatible with Fire Emblem Engage version 2.0.0, but you are running version {}.\nConsider updating your game or uninstalling Cobalt.\n\nCobalt will not run for this play session.", current_ver)
        );
        return;
    }

    let mut loggers: Vec<Box<dyn Log>> = vec![];

    if is_emulator() {
        loggers.push(Box::new(logger::KernelLogger));
    } else {
        loggers.push(Box::new(logger::TcpLogger::new()));
    }

    multi_log::MultiLogger::init(loggers, log::Level::Info).unwrap();

    skyline::install_hooks!(mount_rom_hook, il2cpp_init_hook, socket_initialize_hook, socket_initialize_config_hook);

    // AssetTable table expansion
    Patch::in_text(0x01BAF960).bytes(&[0xE1, 0x03, 0x13, 0x32]).unwrap();
    Patch::in_text(0x01BAF990).bytes(&[0xE1, 0x03, 0x17, 0x32]).unwrap();
    Patch::in_text(0x01BAF9B8).bytes(&[0xE1, 0x03, 0x17, 0x32]).unwrap();
    Patch::in_text(0x01BAF9E0).bytes(&[0xE1, 0x03, 0x17, 0x32]).unwrap();
    Patch::in_text(0x01BAFB2C).bytes(&[0xE2, 0x03, 0x17, 0x32]).unwrap();
    Patch::in_text(0x01BAFBFC).bytes(&[0xE3, 0x03, 0x17, 0x32]).unwrap();
    Patch::in_text(0x01BAFC04).bytes(&[0xE4, 0x03, 0x17, 0x32]).unwrap();
    Patch::in_text(0x01BAFEF0).bytes(&[0x29, 0xFD, 0x3F, 0x11]).unwrap();
    Patch::in_text(0x0211CE04).bytes(&[0x21, 0xFC, 0x3F, 0x11]).unwrap();

    // Skill table expansion

    // MOV w1, #0x2000
    Patch::in_text(0x247DD08).bytes(&[0x01, 0x00, 0x84, 0x52]).unwrap();
    // MOV w1, #0x2000
    Patch::in_text(0x247DE18).bytes(&[0x01, 0x00, 0x84, 0x52]).unwrap();
    // CMP w27, #0x2000
    Patch::in_text(0x249EAB4).bytes(&[0x7F, 0x0B, 0x40, 0x71]).unwrap();
    // CMP w21, #0x2000
    Patch::in_text(0x249ED70).bytes(&[0xBF, 0x0A, 0x40, 0x71]).unwrap();

    // DragonAttributeCommand::Get, fixes the returned value from 8.0 to 16.0
    
    // FMOV s0, 0x41800000
    Patch::in_text(0x21f8420).bytes(&[0x00, 0x10, 0x26, 0x1e]).unwrap();

    println!("Cobalt v{} is installed and running!", cobalt::utils::env::get_cobalt_version());
}
