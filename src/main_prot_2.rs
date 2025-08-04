use sensehat::{SenseHat, Colour};
use serde::Deserialize;
use std::{
    fs,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};
use notify::{RecommendedWatcher, RecursiveMode, Watcher, EventKind, Config as NotifyConfig};

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

fn read_config() -> Option<Config> {
    let config_data = fs::read_to_string("config.json").ok()?;
    serde_json::from_str(&config_data).ok()
}

fn main() {
    // Shared configuration between threads
    let config = Arc::new(Mutex::new(read_config().expect("Failed to load config")));
    println!("Parsed configuration: {:?}", config.lock().unwrap());

    // Watch config.json for changes
    let config_path = "config.json";
    let config_clone = Arc::clone(&config);
    thread::spawn(move || {
        let (tx, rx) = std::sync::mpsc::channel();
        let mut watcher: RecommendedWatcher =
            Watcher::new(tx, NotifyConfig::default()).expect("Failed to create watcher");

        watcher
            .watch(std::path::Path::new(config_path), RecursiveMode::NonRecursive)
            .expect("Failed to watch config.json");

        for res in rx {
            if let Ok(event) = res {
                if matches!(event.kind, EventKind::Modify(_)) {
                    if let Some(new_config) = read_config() {
                        let mut cfg = config_clone.lock().unwrap();
                        *cfg = new_config;
                        println!("Reloaded configuration: {:?}", *cfg);
                    } else {
                        println!("Warning: Could not parse updated config.");
                    }
                }
            }
        }
    });

    let mut sense = SenseHat::new().expect("Failed to initialize Sense HAT");

    loop {
        let temp = sense
            .get_temperature_from_humidity()
            .expect("Failed to read temperature")
            .as_celsius() as f64;

        println!("Current temp: {:.2}°C", temp);

        let config_guard = config.lock().unwrap();
        let mut alert = false;
        for sensor in &config_guard.sensors {
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
        sense.clear().unwrap(); // optional: clear screen after delay
    }
}
