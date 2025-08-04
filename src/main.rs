use sensehat::{SenseHat, Colour};
use serde::Deserialize;
use std::{fs, thread, time::{Duration, Instant}, path::Path};
//use notify::{Watcher, RecursiveMode, watcher, DebouncedEvent};
use notify::{Config as NotifyConfig, RecommendedWatcher, RecursiveMode, Watcher, Event, EventKind};
//use std::sync::mpsc::channel;
//use std::time::Duration;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Debug, Deserialize, Clone)]
struct SensorConfig {
    name: String,
    min: f64,
    max: f64,
}

#[derive(Debug, Deserialize, Clone)]
struct Config {
    sensors: Vec<SensorConfig>,
}

fn hash_config_file(path: &str) -> u64 {
    match fs::read_to_string(path) {
        Ok(content) => {
            let mut hasher = DefaultHasher::new();
            content.hash(&mut hasher);
            hasher.finish()
        }
        Err(_) => 0,
    }
}

fn load_config(path: &str) -> Config {
    let config_data = fs::read_to_string(path)
        .expect("Failed to read config.json");
    serde_json::from_str(&config_data).expect("Invalid JSON config")
}

fn watch_config_file(watcher: &mut RecommendedWatcher, config_path: &str) {
    watcher
        .watch(Path::new(config_path), RecursiveMode::NonRecursive)
        .expect("Failed to watch config.json");
}

fn main() {
    let config_path = "config.json";
    let mut config = load_config(config_path);
    println!("Parsed configuration: {:?}", config);

    let mut sense = SenseHat::new()
        .expect("Failed to initialize Sense HAT");

    // Setup file watcher
    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = RecommendedWatcher::new(
 	move |res| tx.send(res).unwrap(),
    	NotifyConfig::default(),
    ).expect("Failed to create watcher");

    watch_config_file(&mut watcher, config_path);
    //watcher
    //	.watch(Path::new(config_path), RecursiveMode::NonRecursive)
    //	.expect("Failed to watch config.json");
    
    //let mut last_reload = Instant::now() - Duration::from_secs(10); // Initialize to a past time
    let mut config_hash = hash_config_file(config_path);
    loop {
        // Check for updated config
    	if let Ok(Ok(Event { kind, .. })) = rx.recv_timeout(Duration::from_millis(50)) {
	   println!("check here: {:?}", kind); 
      	   let new_hash = hash_config_file(config_path);
	   if new_hash != config_hash {	  
		 //if let Event { kind: EventKind::Modify(_), .. } = event {
            	    println!("Config file changed. Reloading...");
            	    config = load_config(config_path);
            	    println!("Reloaded config: {:?}", config);
		    //last_reload = Instant::now();
   	    	   config_hash = new_hash;
	   }
    	// If the file was removed (e.g. via overwrite), re-register the watcher
    	   if matches!(kind, EventKind::Remove(_)) {
           	println!("File removed or replaced. Re-watching config file...");
           	watch_config_file(&mut watcher, config_path);
    	   }

	}

        // Get current temperature
        let temp = sense
            .get_temperature_from_humidity()
            .expect("Failed to read temperature")
            .as_celsius() as f64;

        println!("Current temp: {:.2}°C", temp);

        let mut alert = false;
        for sensor in &config.sensors {
            if temp < sensor.min || temp > sensor.max {
                println!("{} out of range! Temp: {:.2}", sensor.name, temp);
                alert = true;
                break;
            }
        }
        let (text, fg, bg) = if alert {
            ("NG", Colour::RED, Colour::BLACK)
        } else {
            ("OK", Colour::GREEN, Colour::BLACK)
        };

        sense.text(text, fg, bg).unwrap();
        thread::sleep(Duration::from_secs(5));
        sense.clear().unwrap();
    }
}
