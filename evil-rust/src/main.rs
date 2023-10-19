mod config;
mod wallpaper;

use std::error::Error;
use simple_logger::SimpleLogger;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    SimpleLogger::new().init().unwrap();

    let loaded = config::get_base_config();
    log::info!("Loaded config: {:?}", loaded);

    loop {
        log::debug!("Sleeping...");
        tokio::time::sleep(std::time::Duration::from_secs(loaded.get_main_loop_sleep())).await;
    }

    Ok(())
}
