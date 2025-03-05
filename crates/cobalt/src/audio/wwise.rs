use unity::prelude::*;

use std::{
    str::FromStr,
    ffi::{CStr, CString},
};

use camino::Utf8PathBuf;

#[repr(i32)]
#[derive(PartialEq)]
pub enum AkCallbackType {
    EndOfEvent = 1,
    EndOfDynamicSequenceItem = 2,
    Marker = 4,
    Duration = 8,
    SpeakerVolumeMatrix = 16,
    Starvation = 32,
    MusicPlaylistSelect = 64,
    MusicPlayStarted = 128,
    MusicSyncBeat = 256,
    MusicSyncBar = 512,
    MusicSyncEntry = 1024,
    MusicSyncExit = 2048,
    MusicSyncGrid = 4096,
    MusicSyncUserCue = 8192,
    MusicSyncPoint = 16384,
    MusicSyncAll = 32512,
    MIDIEvent = 65536,
    CallbackBits = 1048575,
    EnableGetSourcePlayPosition = 1048576,
    EnableGetMusicPlayPosition = 2097152,
    EnableGetSourceStreamBuffering = 4194304,
    Monitoring = 536870912,
    Bank = 1073741824,
    AudioInterruption = 570425344,
    AudioSourceChange = 587202560,
}

#[repr(C)]
pub struct WwiseFileHolder {
    filesize: i64,
    unk1: u32,
    unk2: u32,
    unk3: u64,
    handle: skyline::nn::fs::FileHandle,
}

#[skyline::hook(offset = 0x169d700)]
pub fn wwise_file_open_hook(path: *const i8, unk1: u64, _unk2: *const u8, unk3: &mut WwiseFileHolder) -> u32 {
    let str_path = unsafe { CStr::from_ptr(path).to_str().unwrap() };
    let utfpath = Utf8PathBuf::from_str(str_path).unwrap();

    unsafe {
        let relative_path = utfpath.strip_prefix("rom:/").unwrap();

        let manager = mods::manager::Manager::get();

        let res = if let Some(loc) = manager.get_locations().find(|loc| loc == relative_path) {
            let full_path = manager.get_absolute_full_path(loc).unwrap();
            let new_path = CString::new(full_path.as_str()).unwrap();

            skyline::nn::fs::OpenFile(&mut unk3.handle, new_path.as_c_str().to_bytes_with_nul().as_ptr(), 1)
        } else {
            skyline::nn::fs::OpenFile(&mut unk3.handle, path as *const u8, 1)
        };

        if res == 0 {
            skyline::nn::fs::GetFileSize(&mut unk3.filesize, unk3.handle);
            unk3.unk2 = 0;
            unk3.unk3 = unk1;
            1
        } else {
            2
        }
    }
}

#[skyline::hook(offset = 0x24f9010)]
pub fn wwise_set_state(this: *const u8, valuename: &Il2CppString, method_info: OptionalMethod) {
    // println!("StateGroup name: {}", valuename.to_string());
    call_original!(this, valuename, method_info)
}