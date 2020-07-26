use std::collections::HashMap;

pub trait DashDataProvider {
    fn dash_data(&mut self) -> DashData;
}

pub struct DashData {
    inner: HashMap<&'static str, String>,
}

impl DashData {
    pub fn new() -> DashData {
        DashData {
            inner: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: &'static str, value: String) {
        self.inner.insert(key, value);
    }

    pub fn get(&mut self, key: &'static str) -> Option<&String> {
        self.inner.get(key)
    }

    pub fn as_json(&self) -> String {
        format!("{:?}", self.inner)
    }
}
