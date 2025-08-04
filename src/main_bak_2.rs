use sensehat::SenseHat;
use serde::Deserialize;
use std::{fs, thread, time::Duration};

#[derive(Debug, Deserialize)]
struct SensorConfig {
    name: String,
    min: f32,
    max: f32,
}

#[derive(Debug, Deserialize)]
struct Config {
    sensors: Vec<SensorConfig>,
}

fn main() {
    let config_data = fs::read_to_string("config.json").expect("Failed to read config.json");
    let config: Config = serde_json::from_str(&config_data).expect("Invalid JSON config");
    println!("Parsed configuration: {:?}", config);

    let mut sense = SenseHat::new().expect("Sense HAT init failed");

    loop {
        let temp = sense.get_temperature();
        println!("Current temp: {:.2}°C", temp);

        for sensor in &config.sensors {
            if temp < sensor.min || temp > sensor.max {
                println!("{} out of range! Temp: {:.2}", sensor.name, temp);
                sense.show_message("ALERT", 255, 0, 0).ok();
            } else {
                sense.show_message("OK", 0, 255, 0).ok();
            }
        }

        thread::sleep(Duration::from_secs(5));
    }
}
