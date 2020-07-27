use crate::dash_data_provider::{DashData, DashDataProvider};

use rand::Rng;
use std::thread;
use std::time::Duration;

pub struct PsuedoProvider {}

impl DashDataProvider for PsuedoProvider {
    fn dash_data(&mut self) -> DashData {
        let rpm = rand::thread_rng().gen_range(0, 600) + 4500;
        let rpm_string = format!("{}", rpm);

        let gear = 2;
        let gear_string = format!("{}", gear);

        let vss1: f64 = (1.888 * 4.3 * rpm as f64) / 4000_f64;
        let vss1_string = format!("{}", vss1);

        let mut data = DashData::new();
        data.insert("rpm", rpm_string);
        data.insert("vss1", vss1_string);
        data.insert("gear", gear_string);
        data.insert("sensors1", 35.to_string()); // Generic Sensor in MS
        data.insert("sensors2", 60.to_string());
        data.insert("map", 2.to_string());
        data.insert("clt", 175.to_string());
        data.insert("launch", true.to_string());
        data.insert("pit", false.to_string());

        // emulate some read time
        thread::sleep(Duration::from_millis(200));

        data
    }
}
