mod config;
mod wallpaper;

use std::error::Error;
use simple_logger::SimpleLogger;

fn main() -> Result<(), Box<dyn Error>> {
    SimpleLogger::new().init().unwrap();

    let loaded = config::get_config();
    log::info!("Loaded config: {:?}", loaded);

    Ok(())
}
