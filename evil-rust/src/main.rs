mod config;
mod wallpaper;

use std::error::Error;
use std::time::Duration;
use log::LevelFilter;
use simple_logger::SimpleLogger;
use crate::config::ModuleConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    SimpleLogger::new().with_level(get_log_level()).init().unwrap();

    let base_config = config::get_base_config();
    log::info!("Loaded base_config: {:?}", base_config);
    let wallpaper_module = wallpaper::WallpaperModule::new(&base_config);
    log::info!("Loaded wallpaper_module: {:?}", wallpaper_module);

    loop {
        #[cfg(debug_assertions)]
        log::debug!("Sleeping...");
        tokio::time::sleep(Duration::from_secs(base_config.get_main_loop_sleep())).await;
    }

    Ok(())
}

fn get_log_level() -> LevelFilter {
    #[cfg(debug_assertions)]
    return LevelFilter::Debug;
    #[cfg(not(debug_assertions))]
    return LevelFilter::Error;
}