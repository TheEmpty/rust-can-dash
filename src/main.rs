// TODO: unwraps should be protected.
extern crate web_view;
extern crate rand;

#[macro_use]
extern crate log;
extern crate simple_logger;

use web_view::*;
use std::io::prelude::*;
use std::collections::HashMap;
use std::thread;
use std::time::Duration;
use rand::Rng;
use std::time::SystemTime;

// TODO: evaluate if this should be user_data or if update_loop
// can just get and pass the data without this context map.
// Main consideration is stale data and understanding if we will do
// "partial" updates or not.
type CanData = HashMap<&'static str, String>;

static mut odometer: f64 = 0.0;

fn main() {
    simple_logger::init().unwrap();

    let html: String = std::fs::read_to_string("dash.html").unwrap().parse().unwrap();
    unsafe { odometer = std::fs::read_to_string("odometer.txt").unwrap().parse().unwrap(); }
    let view = build_web_view(html);
   
    update_loop(&view);
    inject_user_configuration(&view);

    let run = view.run();
    if run.is_err() {
        error!("Fail to run webview, {:?}", run.unwrap_err());
    }

}

fn build_web_view(html: String) -> WebView<'static, CanData> {
    let can_data: CanData = HashMap::new();
    let mut view = web_view::builder()
        .title("Rust Can Dash")
        .content(Content::Html(html))
        .size(800, 100)
        .resizable(true)
        .debug(true)
        .user_data(can_data)
        .invoke_handler(|webview, arg| {
            match arg {
                "enter" => webview.set_fullscreen(true),
                "exit" => webview.set_fullscreen(false),
                _ => (),
            }
            Ok(())
        })
        .build()
        .unwrap();
    view.set_fullscreen(true);
    view
}

fn inject_user_configuration(source: &WebView<CanData>) {
    let user_configuration: String = std::fs::read_to_string("user_configuration.json").unwrap().parse::<String>().unwrap().replace("\n", "");
    let handle = source.handle();
    // TODO: can't call before 'run'. But having a hard time doing that and keeping rustc happy.
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(500));
        handle.dispatch(move |view| {
            debug!("Updating user_configuration({})", user_configuration);
            view.eval(&format!("user_configuration({})", user_configuration));
            Ok(())
        })
    });
}

fn update_web_view(view: &mut WebView<CanData>) {
    // Mock data for PoC
    let rpm = rand::thread_rng().gen_range(0, 600) + 4500;
    let rpm_string = format!("{}", rpm);

    let gear = 2;
    let gear_string = format!("{}", gear);

    let vss1: i32 = ((1.888 * 4.3 * rpm as f64)/4000 as f64).round() as i32;
    let vss1_string = format!("{}", vss1);

    let user_data = &mut *view.user_data_mut();
    user_data.insert("rpm", rpm_string);
    user_data.insert("vss1", vss1_string);
    user_data.insert("gear", gear_string);
    user_data.insert("fuel", "35".to_string());
    user_data.insert("launch", "true".to_string());
    unsafe { user_data.insert("odometer", format!("{}", odometer)); }
    let update_cycle = view.eval(&format!("update({:?})", view.user_data()));
    if update_cycle.is_err() {
        let error = update_cycle.unwrap_err();
        error!("Failed to update view, {:?}", error);
    } 
}

// This is suuppperrr rough TODO: make odometer less hacky
static mut last_odometer_save: SystemTime = SystemTime::UNIX_EPOCH;
fn update_odometer(vss: f32, time: Duration) {
    let distance_travelled = (vss/3600000000000.0) * time.as_nanos() as f32;
    unsafe {
        odometer = odometer + distance_travelled as f64;
        if SystemTime::now().duration_since(last_odometer_save).unwrap().as_secs() >= 60 {
            // TODO: also on 'drop'
            last_odometer_save = SystemTime::now();
            thread::spawn(move || {
                // save here
                let mut file = std::fs::File::create("odometer.txt").unwrap();
                file.write_all(format!("{}", odometer).as_bytes());
            });
        }
    }
}

static mut last_update_run: SystemTime = SystemTime::UNIX_EPOCH;
fn update_loop(source: &WebView<CanData>) {
    let handle = source.handle();
    unsafe {
        last_odometer_save = SystemTime::now();
        last_update_run = SystemTime::now();
    }
    thread::spawn(move || loop {
        {
            let result = handle.dispatch(move |view| {
                let time_since_last_run: Duration;
                unsafe {
                    time_since_last_run = SystemTime::now().duration_since(last_update_run).unwrap();
                    last_update_run = SystemTime::now();
                }
                update_odometer(10.00, time_since_last_run);
                update_web_view(view);
                Ok(())
            });

            if result.is_err() {
                error!("Failed to dispatch, {:?}", result.unwrap_err());
            }
        }
        thread::sleep(Duration::from_millis(200)); // PoC
    });
}
