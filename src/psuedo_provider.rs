use crate::dash_data_provider::{DashData, DashDataProvider};

use rand::Rng;
use std::thread;
use std::time::Duration;

pub struct PsuedoProvider {}

static PULSES_PER_MILE: f32 = 4000_f32;
static DIFF_RATIO: f32 = 4.3;
static GEAR_RATIOS: [f32; 5] = [
    3.136,
    1.888,
    1.330,
    1.000,
    0.814,
];

static mut RPM: u32 = 800;
static mut GEAR: usize = 1;

static mut FUEL_SWEEP: u8 = 100;

impl DashDataProvider for PsuedoProvider {
    fn dash_data(&mut self) -> DashData {
        let rpm_string;
        let gear_string;
        let vss1;
        let vss1_string;
        let sensors1_string;

        unsafe {
            RPM += rand::thread_rng().gen_range(0, 200);
            if RPM >= 7500 {
                GEAR += 1;
                RPM = 5500;
                if GEAR == 6 {
                    GEAR = 1;
                    RPM = 800;
                }
            }
            vss1 = (RPM*60) as f32 / 877.92 / (GEAR_RATIOS[GEAR-1] * DIFF_RATIO);

            rpm_string = format!("{}", RPM);
            gear_string = format!("{}", GEAR);
            vss1_string = format!("{}", vss1);

            if FUEL_SWEEP == 0 {
                FUEL_SWEEP = 100;
            } else {
                FUEL_SWEEP -= 1;
            }
            sensors1_string = format!("{}", FUEL_SWEEP);
        }

        let mut data = DashData::new();
        data.insert("rpm", rpm_string);
        data.insert("vss1", vss1_string);
        data.insert("gear", gear_string);
        data.insert("sensors1", sensors1_string); // Generic Sensor in MS
        data.insert("sensors2", 60.to_string());
        data.insert("map", 2.to_string());
        data.insert("clt", 175.to_string());
        data.insert("iat", 70.to_string());
        data.insert("launch", (vss1 <= 10_f32).to_string());
        data.insert("pit-limiter", false.to_string());

        // emulate some read time
        thread::sleep(Duration::from_millis(200));

        data
    }
}
