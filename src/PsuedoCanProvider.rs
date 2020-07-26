use crate::CanProvider::{CanData, CanProvider};

use rand::Rng;
use std::collections::HashMap;
use std::thread;
use std::time::Duration;

pub struct PsuedoCanProvider {}

impl CanProvider for PsuedoCanProvider {
    fn can_data(&mut self) -> CanData {
        let rpm = rand::thread_rng().gen_range(0, 600) + 4500;
        let rpm_string = format!("{}", rpm);
    
        let gear = 2;
        let gear_string = format!("{}", gear);
    
        let vss1: i32 = ((1.888 * 4.3 * rpm as f64)/4000 as f64).round() as i32;
        let vss1_string = format!("{}", vss1);

        let mut can_data: CanData = HashMap::new();
        can_data.insert("rpm", rpm_string);
        can_data.insert("vss1", vss1_string);
        can_data.insert("gear", gear_string);
        can_data.insert("sensors1", 35.to_string()); // Generic Sensor in MS
        can_data.insert("launch", true.to_string());

        // emulate some read time
        thread::sleep(Duration::from_millis(200));

        can_data
    }
}