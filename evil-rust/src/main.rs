mod config;
mod wallpaper;
mod module;

use std::error::Error;
use std::thread::sleep;
use std::time::Duration;
use log::LevelFilter;
use simple_logger::SimpleLogger;
use crate::config::ModuleConfig;
use crate::module::Module;

// #[tokio::main]
fn main() -> Result<(), Box<dyn Error>> {
    SimpleLogger::new().with_level(get_log_level()).init().unwrap();

    // Only continue if we have admin rights
    if !is_elevated::is_elevated() {
        log::error!("Not running as Admin");
        return Ok(());
    }

    let base_config = config::get_base_config();
    log::info!("Loaded base_config: {:?}", base_config);
    let mut wallpaper_module = wallpaper::WallpaperModule::new(&base_config);
    log::info!("Loaded wallpaper_module: {:?}", wallpaper_module);

    loop {
        // TODO maybe make async? Since it may take while to run it
        wallpaper_module.trigger();
        // tokio::time::sleep(Duration::from_secs(base_config.get_main_loop_sleep())).await;
        sleep(Duration::from_secs(base_config.get_main_loop_sleep()));
    }

    Ok(())
}

fn get_log_level() -> LevelFilter {
    // #[cfg(debug_assertions)]
    return LevelFilter::Debug;
    // #[cfg(not(debug_assertions))]
    // return LevelFilter::Error;
}