// TODO: odmeter save on exit
// TODO: support multiple dashes
// TODO: support external buttons/triggers

extern crate rand;
extern crate web_view;

mod dash_data_provider;
mod odometer;
mod psuedo_provider;

use dash_data_provider::{DashData, DashDataProvider};
use odometer::Odometer;
use psuedo_provider::PsuedoProvider;

#[macro_use]
extern crate log;
extern crate simple_logger;

use std::thread;
use std::time::Duration;
use std::time::SystemTime;
use web_view::*;

fn main() {
    simple_logger::init().expect("Failed to setup logger");

    let html: String = std::fs::read_to_string("dash.html")
        .expect("Unable to load dash.html")
        .parse()
        .expect("Unable to convert dash.html to a string");
    let view = build_web_view(html);

    // TODO: provider should be create here, and based on ecu_configuration.json
    update_loop(&view);
    inject_dash_configuration(&view);

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
        .expect("Failed to build_web_view");
    view.set_fullscreen(true);
    view
}

fn inject_dash_configuration(source: &WebView<()>) {
    let dash_configuration: String = std::fs::read_to_string("dash_configuration.json")
        .expect("Failed to read dash_configuration.json")
        .parse::<String>()
        .expect("Failed to convert dash_configuration.json to a string")
        .replace("\n", "");
    let handle = source.handle();
    // TODO: can't call before 'run'. But having a hard time doing that and keeping rustc happy.
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(500));
        handle.dispatch(move |view| {
            debug!("Updating dash_configuration({})", dash_configuration);
            view.eval(&format!("dash_configuration({})", dash_configuration))
        })
    });
}

fn update_web_view(view: &mut WebView<()>, can_data: &DashData) {
    let java_script = format!("update({})", can_data.as_json());
    let update_cycle = view.eval(&java_script);
    if update_cycle.is_err() {
        let error = update_cycle.unwrap_err();
        error!("Failed to update view, {:?}", error);
    }
}

static mut LAST_UPDATE_RUN: SystemTime = SystemTime::UNIX_EPOCH;

static mut ODOMETER: Odometer = Odometer {
    odometer: 0_f64,
    last_save: SystemTime::UNIX_EPOCH,
};

fn update_loop(source: &WebView<()>) {
    let handle = source.handle();
    unsafe {
        ODOMETER.odometer = std::fs::read_to_string("odometer.txt")
            .expect("Failed to find odometer.txt")
            .parse()
            .expect("Failed to convert odometer.txt to f64");

        LAST_UPDATE_RUN = SystemTime::now();
    }

    thread::spawn(move || {
        let provider: &mut dyn DashDataProvider = &mut PsuedoProvider {};

        loop {
            let mut dash_data = provider.dash_data();

            let result = handle.dispatch(move |view| {
                let time_since_last_run: Duration;
                unsafe {
                    time_since_last_run = SystemTime::now()
                        .duration_since(LAST_UPDATE_RUN)
                        .expect("Whoops, time ran backwards");
                    LAST_UPDATE_RUN = SystemTime::now();
                }

                let odometer_reading: f64;
                unsafe {
                    odometer_reading = ODOMETER.update(
                        dash_data.get("vss1").unwrap().parse().unwrap(),
                        time_since_last_run,
                    );
                    ODOMETER.auto_save();
                }
                dash_data.insert("odometer", odometer_reading.to_string());
                update_web_view(view, &dash_data);
                Ok(())
            });

            if result.is_err() {
                error!("Failed to dispatch, {:?}", result.unwrap_err());
            }
        }
    });
}
