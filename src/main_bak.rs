use sensehat::{SenseHat, LedColor};
//use sense_hat::LedColor;
use serde::Deserialize;
use std::{fs, thread, time::Duration};

#[derive(Debug, Deserialize)]
struct SensorConfig {
    name: String,
    min: f32,
    max: f32,
    led_color: Option<String>, // Optional custom color
}

#[derive(Debug, Deserialize)]
struct Config {
    sensors: Vec<SensorConfig>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config_data = fs::read_to_string("config.json")?;
    let config: Config = serde_json::from_str(&config_data)?;
    println!("Parsed configuration: {:?}", config);

    let mut sense = SenseHat::new()?;

    loop {
        let temp = sense.temperature()?;
        println!("Current temp: {:.2}°C", temp);

        for sensor in &config.sensors {
            if temp < sensor.min || temp > sensor.max {
                println!("{} out of range! Temp: {:.2}", sensor.name, temp);
                sense.clear(LedColor::from_rgb(255, 0, 0))?; // red alert
            } else {
                sense.clear(LedColor::from_rgb(0, 255, 0))?; // green safe
            }
        }

        thread::sleep(Duration::from_secs(5));
    }
}
