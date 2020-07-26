use std::io::prelude::*;
use std::time::Duration;
use std::time::SystemTime;

#[derive(Copy, Clone)]
pub struct Odometer {
    pub odometer: f64,         // TODO: only pub help init in main.rs
    pub last_save: SystemTime, // TODO: above
}

impl Odometer {
    pub fn new() -> Self {
        Odometer {
            odometer: std::fs::read_to_string("odometer.txt")
                .expect("Failed to find odometer.txt")
                .parse()
                .expect("Failed to convert odometer.txt to f64"),
            last_save: SystemTime::now(),
        }
    }

    pub fn update(&mut self, vss: f32, time: Duration) -> f64 {
        let distance_travelled = (vss / 3600000000000_f32) * time.as_nanos() as f32;
        self.odometer += distance_travelled as f64;
        self.odometer
    }

    pub fn auto_save(&mut self) {
        if SystemTime::now()
            .duration_since(self.last_save)
            .unwrap()
            .as_secs()
            >= 60
        {
            self.save();
        }
    }

    pub fn save(&mut self) {
        self.last_save = SystemTime::now();
        let mut file =
            std::fs::File::create("odometer.txt").expect("Failed to open odometer.txt for writing");
        let result = file.write_all(format!("{}", self.odometer).as_bytes());
        if result.is_err() {
            error!("Failed to write to odometer.txt {:?}", result);
        }
    }
}
