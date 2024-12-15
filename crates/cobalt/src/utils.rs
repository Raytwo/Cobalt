#![allow(dead_code)]

pub mod env {
    use std::sync::LazyLock;

    #[non_exhaustive]
    pub enum RunEnvironment {
        Switch,
        Emulator,
    }

    static PLATFORM: LazyLock<RunEnvironment> = LazyLock::new(|| {
        let base_addr = unsafe { skyline::hooks::getRegionAddress(skyline::hooks::Region::Text) as u64 };

        if base_addr == 0x8004000 || base_addr == 0x8504000 {
            RunEnvironment::Emulator
        } else {
            RunEnvironment::Switch
        }
    });

    pub fn get_running_env() -> &'static RunEnvironment {
        &PLATFORM
    }

    pub fn is_hardware() -> bool {
        matches!(get_running_env(), RunEnvironment::Switch)
    }

    pub fn is_emulator() -> bool {
        matches!(get_running_env(), RunEnvironment::Emulator)
    }

    pub fn get_cobalt_version<'a>() -> &'a str {
        env!("CARGO_PKG_VERSION")
    }
}

pub mod paths {
    use std::io;

    use camino::Utf8PathBuf;

    pub fn ensure_paths_exist() -> io::Result<()> {
        std::fs::create_dir_all(mods())?;
        std::fs::create_dir_all(config())?;
        Ok(())
    }

    pub fn mods() -> Utf8PathBuf {
        Utf8PathBuf::from("sd:/engine/mods")
    }

    pub fn config() -> Utf8PathBuf {
        Utf8PathBuf::from("sd:/engine/config")
    }
}
