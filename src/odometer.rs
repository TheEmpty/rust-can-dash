use std::time::SystemTime;
use std::time::Duration;
use std::io::prelude::*;

#[derive(Copy, Clone)]
pub struct Odometer {
    odometer: f64,
    last_save: SystemTime,
}

impl Odometer {
    pub fn new() -> Self {
        Odometer {
            odometer: std::fs::read_to_string("odometer.txt").unwrap().parse().unwrap(),
            last_save: SystemTime::now()
        }
    }

    pub fn update(&mut self, vss: f32, time: Duration) -> f64 {
        let distance_travelled = (vss/3600000000000.0) * time.as_nanos() as f32;
        self.odometer += distance_travelled as f64;
        self.odometer
    }

    pub fn auto_save(&mut self) {
        if SystemTime::now().duration_since(self.last_save).unwrap().as_secs() >= 60 {
            self.save();
        }
    }

    pub fn save(&mut self) {
        self.last_save = SystemTime::now();
        let mut file = std::fs::File::create("odometer.txt").unwrap();
        file.write_all(format!("{}", self.odometer).as_bytes());
    }
}

pub struct DropableOdometer {
    pub inner: Odometer
}

impl DropableOdometer {
    pub fn keep(&mut self) -> Result<(), ()> {
        Ok(())
    }
}

impl Drop for DropableOdometer {
    fn drop(&mut self) {
        debug!("DropableOdometer, dropped.");
        self.inner.save();
    }
}
