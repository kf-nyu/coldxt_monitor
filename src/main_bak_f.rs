use sensehat::SenseHat;
use sensehat_screen::Screen;
use tint::{Colour, Framebuffer};
//use sensehat_screen::framebuffer::Draw;
use font8x8::UnicodeFonts;
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

fn draw_char(screen: &mut Screen, ch: char, color: Colour) {
    screen.clear().unwrap();

    if let Some(char_bitmap) = font8x8::BASIC_FONTS.get(ch) {
        for (y, row) in char_bitmap.iter().enumerate() {
            for x in 0..8 {
                if (row >> x) & 1 == 1 {
                    // Sense HAT screen origin is top-left; x/y flipped vs font
                    screen.set_pixel(7 - x, y as u8, color).unwrap();
                }
            }
        }
    }
}

fn main() {
    let config_data = fs::read_to_string("config.json").expect("Failed to read config.json");
    let config: Config = serde_json::from_str(&config_data).expect("Invalid JSON config");
    println!("Parsed configuration: {:?}", config);

    let mut sense = SenseHat::new().expect("Failed to initialize Sense HAT");

    loop {
        // get_temperature_from_humidity returns Result<Temperature, SenseHatError>
        let temp = sense
            .get_temperature_from_humidity()
            .expect("Failed to read temperature")
            .as_celsius() as f64; // get the actual f32

        println!("Current temp: {:.2}°C", temp);

        let mut alert = false;

        for sensor in &config.sensors {
            if temp < sensor.min || temp > sensor.max {
                println!("{} out of range! Temp: {:.2}", sensor.name, temp);
                alert = true;
                break;
            }
        }
	let mut screen = Screen::open("/dev/fb0").expect("Failed to open screen");
        if alert {
    		draw_char(&mut screen, 'X', Colour { red: 255.0, green: 0.0, blue: 0.0, alpha: 1.0, }); // red "X"
	} else {
    		draw_char(&mut screen, 'O', Colour { red: 0.0, green: 255.0, blue: 0.0, alpha: 1.0, }); // green "O"
	}
	thread::sleep(Duration::from_secs(5));
    }
}
