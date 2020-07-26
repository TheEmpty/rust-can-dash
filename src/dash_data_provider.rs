use std::collections::HashMap;

pub type DashData = HashMap<&'static str, String>;

pub trait DashDataProvider: std::marker::Sync {
    fn dash_data(&mut self) -> DashData;
}
