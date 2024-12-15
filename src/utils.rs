#![allow(dead_code)]

pub mod env {
    use std::str::FromStr;
    use std::sync::LazyLock;
    use semver::Version;

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

    /// Wrapper function for getting the version string of the game from nnSdk
    pub fn get_game_version() -> Version {
        unsafe {
            // TODO: Implement this in nnsdk-rs
            let mut version_string = skyline::nn::oe::DisplayVersion { name: [0x00; 16] };
            skyline::nn::oe::GetDisplayVersion(&mut version_string);
            Version::from_str(&skyline::from_c_str(version_string.name.as_ptr())).expect("Smash's version should parse as a proper semver.")
        }
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
        Utf8PathBuf::from("sd:/engage/mods")
    }

    pub fn config() -> Utf8PathBuf {
        Utf8PathBuf::from("sd:/engage/config")
    }
}
