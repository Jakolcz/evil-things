extern crate winapi;

use std::collections::HashMap;
use std::ffi::CString;
use std::io::{Error, ErrorKind};
use std::ops::{Add, Range};
use std::path::PathBuf;
use std::time::{Duration, SystemTime};
use rand::Rng;
use serde::{Deserialize, Serialize};
use winreg::enums::KEY_SET_VALUE;
use winreg::RegKey;
use crate::config::{BaseConfig, load_config, MINUTE, ModuleConfig, save_module_config, SECOND};
#[cfg(not(debug_assertions))]
use crate::config::MAN_DAY;
use crate::module::Module;

pub const MODULE_NAME: &str = "syssound";
const GITHUB_ROOT: &str = "https://raw.githubusercontent.com/Jakolcz/evil-things/main/evil-rust/src/syssound/data/";
const REGISTRY_ROOT: &str = "AppEvents\\Schemes\\Apps\\.Default";
const REGISTRY_CURRENT_KEY: &str = ".Current";


#[derive(Debug, Serialize, Deserialize)]
pub struct SysSoundModule {
    changed: bool,
    trigger_enabled: bool,
    sounds_dir: PathBuf,
    source_http: String,
    frequency_range: Range<u32>,
    sound_mappings: HashMap<String, String>,
    next_trigger: SystemTime,
}

impl ModuleConfig for SysSoundModule {
    fn new(base_config: &BaseConfig) -> Self {
        let module_home = SysSoundModule::construct_module_home(base_config.get_home_dir());

        #[cfg(debug_assertions)]
        let frequency_range = SECOND..10 * SECOND;
        #[cfg(not(debug_assertions))]
        let frequency_range = MINUTE..MAN_DAY;

        load_config(&module_home, MODULE_NAME).unwrap_or_else(|_| {
            let default = Self {
                changed: false,
                trigger_enabled: true,
                sounds_dir: module_home,
                source_http: String::from(GITHUB_ROOT),
                frequency_range: frequency_range.clone(),
                sound_mappings: default_sound_mappings(),
                next_trigger: SystemTime::now().add(Duration::from_secs(rand::thread_rng().gen_range(frequency_range) as u64)),
            };

            save_module_config(&default).unwrap_or_else(|e| {
                log::error!("Error saving config file: {}", e);
            });

            default
        })
    }

    fn refresh_base_config(&mut self, base_config: &BaseConfig) {
        self.sounds_dir = SysSoundModule::construct_module_home(base_config.get_home_dir());
        self.persist();
    }

    fn get_module_name(&self) -> &str {
        MODULE_NAME
    }

    fn get_module_home(&self) -> &PathBuf {
        &self.sounds_dir
    }

    fn get_enabled(&self) -> bool {
        self.trigger_enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.trigger_enabled = enabled;
        self.persist();
    }

    fn construct_module_home(base_home_path: &PathBuf) -> PathBuf {
        base_home_path.join(MODULE_NAME)
    }
}

fn default_sound_mappings() -> HashMap<String, String> {
    HashMap::from([
        (String::from(".Default"), String::from("onii-chan.wav")),
        (String::from("DeviceConnect"), String::from("kimochi.wav")),
        (String::from("DeviceDisconnect"), String::from("uwu.wav")),
        (String::from("DeviceFail"), String::from("ara-ara.wav")),
        (String::from("LowBatteryAlarm"), String::from("turtle.wav")),
        (String::from("Maximize"), String::from("ara-ara.wav")),
        (String::from("Minimize"), String::from("uwu.wav")),
        (String::from("SystemAsterisk"), String::from("onii-chan.wav")),
        (String::from("WindowsLogon"), String::from("dobre-rano.wav")),
        (String::from("WindowsUAC"), String::from("kimochi.wav")),
    ])
}

impl Module for SysSoundModule {
    fn trigger(&mut self) {
        if !self.trigger_enabled {
            log::debug!("SysSoundModule is disabled");
            return;
        }

        if !self.changed {
            self.change_sounds();
            self.changed = true;
            self.persist();
        }

        let play_now = self.next_trigger.lt(&SystemTime::now());
        if !play_now {
            log::debug!("Not playing sounds, next sound in: {:?}", self.next_trigger);
            return;
        }

        log::debug!("Triggering SysSoundModule");
        // self.next_trigger = SystemTime::now().add(Duration::from_secs(rand::thread_rng().gen_range(self.frequency_range) as u64));
        // self.change_sounds();
    }
}

impl SysSoundModule {
    pub fn change_sounds(&mut self) {
        log::debug!("Changing sounds");
        let hkcu = RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
        let sound_root = match hkcu.open_subkey(REGISTRY_ROOT) {
            Ok(key) => key,
            Err(e) => {
                log::error!("Failed to open registry key {}: {}", REGISTRY_ROOT, e);
                return;
            }
        };

        for (event_registry_name, sound) in self.sound_mappings.iter() {
            let current_sound_reg_key = match self.get_sound_reg_key(&sound_root, event_registry_name) {
                Ok(key) => key,
                Err(e) => {
                    log::error!("Failed to open sound event registry key {}: {}", event_registry_name, e);
                    continue;
                }
            };

            if let Err(e) = self.do_sound_change(&current_sound_reg_key, &self.get_full_sound_path(sound)) {
                log::error!("Failed to change sound for {}: {}", event_registry_name, e);
            }
        }
    }

    fn do_sound_change(&self, sound_reg_key: &RegKey, full_sound_path: &PathBuf) -> Result<(), Error> {
        log::debug!("Changing sound {:?} to: {:?}", sound_reg_key, full_sound_path);
        if !full_sound_path.exists() {
            log::error!("Sound path does not exist: {:?}", full_sound_path);
            return Err(Error::new(ErrorKind::NotFound, "Sound path does not exist"));
        }

        let sound_path_str = match full_sound_path.to_str() {
            Some(path) => path,
            None => {
                log::error!("Failed to convert sound path to string: {:?}", full_sound_path);
                return Err(Error::new(ErrorKind::Other, "Failed to convert sound path to string"));
            }
        };

        sound_reg_key.set_value("", &sound_path_str)
    }

    fn get_sound_reg_key(&self, sound_root: &RegKey, event_registry_name: &str) -> Result<RegKey, Error> {
        sound_root.open_subkey(event_registry_name)?
            .open_subkey_with_flags(REGISTRY_CURRENT_KEY, KEY_SET_VALUE)
    }

    fn get_full_sound_path(&self, sound: &str) -> PathBuf {
        self.sounds_dir.join(sound)
    }
}