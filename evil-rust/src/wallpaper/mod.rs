use std::fs;
use std::ops::Range;
use std::path::PathBuf;
use rand::Rng;
use serde::{Deserialize, Serialize};
use crate::config::{BaseConfig, load_config, MINUTE, ModuleConfig, save_module_config};
#[cfg(not(debug_assertions))]
use crate::config::MAN_DAY;
use crate::module::Module;

pub const MODULE_NAME: &str = "wallpaper";
const DEFAULT_SOURCE_HTTP: &str = "https://source.unsplash.com/random/1920x1080";
const GITHUB_ROOT: &str = "https://raw.githubusercontent.com/Jakolcz/evil-things/main/evil-rust/src/wallpaper/data/";
const DEFAULT_WINDOWS_FOLDER: &str = "C:\\Windows\\Web";
const REGISTRY_KEY: &str = "Control Panel\\Desktop";

#[derive(Debug, Serialize, Deserialize)]
pub struct WallpaperModule {
    enabled: bool,
    wallpaper_dir: PathBuf,
    source_http: String,
    frequency_range: Range<u32>,
    original_wallpaper: Option<String>,
}

impl ModuleConfig for WallpaperModule {
    fn new(base_config: &BaseConfig) -> Self {
        let module_home = WallpaperModule::construct_module_home(base_config.get_home_dir());
        load_config(&module_home, MODULE_NAME).unwrap_or_else(|_| {
            let default = Self {
                enabled: true,
                wallpaper_dir: module_home,
                source_http: String::from(GITHUB_ROOT),
                original_wallpaper: None,
                #[cfg(debug_assertions)]
                frequency_range: (MINUTE..2 * MINUTE),
                #[cfg(not(debug_assertions))]
                frequency_range: (MINUTE..MAN_DAY),
            };

            save_module_config(&default).unwrap_or_else(|e| {
                log::error!("Error saving config file: {}", e);
            });

            default
        })
    }

    fn refresh_base_config(&mut self, base_config: &BaseConfig) {
        self.wallpaper_dir = WallpaperModule::construct_module_home(base_config.get_home_dir());
        self.persist();
    }

    fn get_module_name(&self) -> &str {
        MODULE_NAME
    }

    fn get_module_home(&self) -> &PathBuf {
        &self.wallpaper_dir
    }

    fn get_enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        self.persist();
    }

    fn construct_module_home(base_home_path: &PathBuf) -> PathBuf {
        base_home_path.join(MODULE_NAME)
    }
}

impl Module for WallpaperModule {
    fn trigger(&mut self) {
        log::info!("Triggering wallpaper module");
        self.ensure_files_exist();
        self.switch_wallpaper();
    }
}

impl WallpaperModule {
    pub fn get_next_frequency(&self) -> u32 {
        rand::thread_rng().gen_range(self.frequency_range.clone())
    }

    pub fn get_frequency_range(&self) -> Range<u32> {
        self.frequency_range.clone()
    }

    fn set_original_wallpaper(&mut self, original_wallpaper: String) {
        self.original_wallpaper = Some(original_wallpaper);
        self.persist();
    }

    fn get_original_wallpaper(&self) -> Option<String> {
        self.original_wallpaper.clone()
    }

    // #[cfg(debug_assertions)]
    fn switch_wallpaper(&mut self) {
        // log::warn!("Not switching wallpaper in debug mode");
        let hkcu = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
        let desktop = hkcu.open_subkey_with_flags(REGISTRY_KEY, winreg::enums::KEY_ALL_ACCESS).unwrap();
        // let desktop = hkcu.open_subkey(REGISTRY_KEY).unwrap();
        if self.get_original_wallpaper().is_none() {
            let wallpaper_reg_value: String = desktop.get_value("Wallpaper").unwrap();
            log::debug!("Current wallpaper: {}", wallpaper_reg_value);
            self.set_original_wallpaper(wallpaper_reg_value.clone());
        }
    }

    fn ensure_files_exist(&self) {
        let files_exist = fs::read_dir(&self.wallpaper_dir).unwrap().map(|entry| {
            entry.unwrap()
        }).any(|dir_entry| {
            log::debug!("Checking if file {:?} ends with .jpg", dir_entry.path());
            dir_entry.path().to_str().unwrap().ends_with(".jpg")
        });

        if !files_exist {
            log::debug!("Files do not exist in wallpaper dir, downloading");
            self.download_files();
        }
    }

    fn download_files(&self) {
        for i in 0..10 {
            let url = format!("{}{}.jpg", self.source_http, i);
            log::debug!("Downloading file from url: {}", url);
            let response = reqwest::blocking::get(url.clone());
            if response.is_err() {
                log::error!("Error downloading file from url: {}", url);
                break;
            }
            let response = response.unwrap();
            if !response.status().is_success() {
                log::error!("Error downloading file from url: {}", url);
                break;
            }
            let mut dest = self.wallpaper_dir.clone();
            dest.push(format!("{}.jpg", i));
            let mut out = fs::File::create(dest.clone()).unwrap();
            let content = response.bytes().unwrap();
            std::io::copy(&mut content.as_ref(), &mut out).unwrap();
            log::debug!("Downloaded file from to : {}", dest.to_str().unwrap());
        }
    }
}