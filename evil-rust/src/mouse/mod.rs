//! Mouse module.
//!
//! Module for decreasing mouse sensitivity.

use std::ops::Add;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};
use serde::{Deserialize, Serialize};
use crate::config::{load_config, save_module_config, BaseConfig, ModuleConfig, DAY, SECOND};
use crate::module::Module;

pub const MODULE_NAME: &str = "mouse";

#[derive(Debug, Serialize, Deserialize)]
pub struct MouseModule {
    enabled: bool,
    next_trigger: SystemTime,
    frequency: u32,
    module_home: PathBuf,
}

impl ModuleConfig for MouseModule {
    fn new(base_config: &BaseConfig) -> Self {
        let module_home = MouseModule::construct_module_home(base_config.get_home_dir());

        #[cfg(debug_assertions)]
        let change_frequency = 5 * SECOND;
        #[cfg(not(debug_assertions))]
        let change_frequency = 2 * DAY;

        load_config(&module_home, MODULE_NAME).unwrap_or_else(|_| {
            let default = Self {
                enabled: true,
                frequency: change_frequency,
                next_trigger: SystemTime::now(),
                module_home: module_home.clone(),
            };

            save_module_config(&default).unwrap_or_else(|e| {
                log::error!("Error saving config file: {}", e);
            });

            default
        })
    }

    fn refresh_base_config(&mut self, base_config: &BaseConfig) {
        self.module_home = MouseModule::construct_module_home(base_config.get_home_dir());
        self.persist();
    }

    fn get_module_name(&self) -> &str {
        MODULE_NAME
    }

    fn get_module_home(&self) -> &PathBuf {
        &self.module_home
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

impl Module for MouseModule {
    fn trigger(&mut self) {
        if !self.enabled {
            return;
        }

        // TODO SPI_SETWHEELSCROLLLINES

        let change_now = self.get_next_change().lt(&SystemTime::now());
        if !change_now {
            log::debug!("Not decreasing mouse sensitivity, next change in: {:?}", self.get_next_change());
            return;
        }

        self.decrease_sensitivity_and_reschedule();
    }
}

impl MouseModule {
    pub fn decrease_sensitivity_and_reschedule(&mut self) {
        self.decrease_sensitivity();
        self.set_next_change(SystemTime::now().add(Duration::from_secs(self.frequency as u64)));
    }

    pub fn decrease_sensitivity(&mut self) {
        self.get_sensitivity().and_then(|sensitivity| {
            // If sensitivity is already at the minimum, return early
            if sensitivity <= 1 {
                return Ok(());
            }

            log::debug!("Decreasing mouse sensitivity from {} to {}", sensitivity, sensitivity - 1);

            self.set_sensitivity(sensitivity - 1)
        }).unwrap_or_else(|e| {
            log::error!("Error decreasing sensitivity: {}", e);
        });
    }

    fn get_sensitivity(&self) -> Result<u32, String> {
        let mut sensitivity: u32 = 0;

        /*
        Getting (SPI_GETMOUSESPEED): You pass a pointer to a value, so you need two casts—first to convert the reference to a raw pointer, and then to convert it to a *mut c_void pointer.
         */
        unsafe {
            let result = winapi::um::winuser::SystemParametersInfoW(
                winapi::um::winuser::SPI_GETMOUSESPEED,
                0,
                &mut sensitivity as *mut _ as *mut _,
                0,
            );

            // If the function succeeds, the return value is a nonzero value.
            if result == 0 {
                return Err(String::from("Error getting mouse speed"));
            }
        }

        Ok(sensitivity)
    }

    fn set_sensitivity(&self, sensitivity: u32) -> Result<(), String> {
        if sensitivity > 20 {
            return Err(String::from("Sensitivity must be between 0 and 20"));
        }

        /*
        Setting (SPI_SETMOUSESPEED): You pass a value, so you only need one cast.
         */
        unsafe {
            let result = winapi::um::winuser::SystemParametersInfoW(
                winapi::um::winuser::SPI_SETMOUSESPEED,
                0,
                sensitivity as *mut _,
                winapi::um::winuser::SPIF_UPDATEINIFILE | winapi::um::winuser::SPIF_SENDCHANGE,
            );

            // If the function succeeds, the return value is a nonzero value.
            if result == 0 {
                return Err(String::from("Error setting mouse speed"));
            }
        }

        Ok(())
    }

    fn set_next_change(&mut self, next_change: SystemTime) {
        self.next_trigger = next_change;
        self.persist();
    }

    fn get_next_change(&self) -> SystemTime {
        self.next_trigger
    }
}