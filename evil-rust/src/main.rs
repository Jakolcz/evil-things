mod config;
mod wallpaper;

use std::error::Error;
use simple_logger::SimpleLogger;
use crate::config::ModuleConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    SimpleLogger::new().init().unwrap();

    let base_config = config::get_base_config();
    log::info!("Loaded base_config: {:?}", base_config);
    let wallpaper_module_config = wallpaper::WallpaperConfig::new(&base_config);
    log::info!("Loaded wallpaper_module_config: {:?}", wallpaper_module_config);

    loop {
        log::debug!("Sleeping...");
        tokio::time::sleep(std::time::Duration::from_secs(base_config.get_main_loop_sleep())).await;
    }

    Ok(())
}
