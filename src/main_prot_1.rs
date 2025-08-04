use sensehat::{SenseHat, Colour};
use serde::Deserialize;
use std::{fs, thread, time::Duration};

#[derive(Debug, Deserialize)]
struct SensorConfig {
    name: String,
    min: f64,
    max: f64,
}

#[derive(Debug, Deserialize)]
struct Config {
    sensors: Vec<SensorConfig>,
}

fn main() {
    let config_data = fs::read_to_string("config.json")
        .expect("Failed to read config.json");
    let config: Config = serde_json::from_str(&config_data)
        .expect("Invalid JSON config");
    println!("Parsed configuration: {:?}", config);

    let mut sense = SenseHat::new()
        .expect("Failed to initialize Sense HAT");

    loop {
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
        sense.clear().unwrap(); // optional: clear screen after delay
    }
}
