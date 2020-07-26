// TODO: unwraps should be protected.
extern crate web_view;
extern crate rand;
mod PsuedoCanProvider;
mod CanProvider;

#[macro_use]
extern crate log;
extern crate simple_logger;

use web_view::*;
use std::io::prelude::*;
use std::collections::HashMap;
use std::thread;
use std::time::Duration;

use std::time::SystemTime;

static mut odometer: f64 = 0.0;

fn main() {
    simple_logger::init().unwrap();

    let html: String = std::fs::read_to_string("dash.html").unwrap().parse().unwrap();
    unsafe { odometer = std::fs::read_to_string("odometer.txt").unwrap().parse().unwrap(); }
    let view = build_web_view(html);

    // TODO: provider should be create here, and based on config
    update_loop(&view);
    inject_user_configuration(&view);

    let run = view.run();
    if run.is_err() {
        error!("Fail to run webview, {:?}", run.unwrap_err());
    }

}

fn build_web_view(html: String) -> WebView<'static, ()> {
    let mut view = web_view::builder()
        .title("Rust Can Dash")
        .content(Content::Html(html))
        .size(800, 100)
        .resizable(true)
        .debug(true)
        .user_data(())
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

fn inject_user_configuration(source: &WebView<()>) {
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

fn update_web_view(view: &mut WebView<()>, can_data: &HashMap<&str, String>) {
    let java_script = format!("update({:?})", can_data);
    let update_cycle = view.eval(&java_script);
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

// TODO: unsafe { user_data.insert("odometer", format!("{}", odometer)); }
static mut last_update_run: SystemTime = SystemTime::UNIX_EPOCH;
fn update_loop(source: &WebView<()>) {
    let handle = source.handle();
    unsafe {
        last_odometer_save = SystemTime::now();
        last_update_run = SystemTime::now();
    }
    thread::spawn(move || loop {
        let provider: &mut dyn CanProvider::CanProvider = &mut PsuedoCanProvider::PsuedoCanProvider { };
        let mut can_data = provider.can_data();

        let result = handle.dispatch(move |view| {
            let time_since_last_run: Duration;
            unsafe {
                time_since_last_run = SystemTime::now().duration_since(last_update_run).unwrap();
                last_update_run = SystemTime::now();
            }

            update_odometer(can_data.get("vss1").unwrap().parse().unwrap(), time_since_last_run);
            unsafe { can_data.insert("odometer", odometer.to_string()); }
            update_web_view(view, &can_data);
            Ok(())
        });

        if result.is_err() {
            error!("Failed to dispatch, {:?}", result.unwrap_err());
        }
    });
}
