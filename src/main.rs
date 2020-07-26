// TODO: unwraps should be protected.
// TODO: odmeter save on exit
// TODO: support multiple dashes
// TODO: support external buttons/triggers

extern crate web_view;
extern crate rand;

mod psuedo_can_provider;
mod can_provider;
mod odometer;

use psuedo_can_provider::PsuedoCanProvider;
use odometer::Odometer;
use can_provider::CanProvider;

#[macro_use]
extern crate log;
extern crate simple_logger;

use web_view::*;
use std::collections::HashMap;
use std::thread;
use std::time::Duration;

use std::time::SystemTime;

fn main() {
    simple_logger::init().expect("Failed to setup logger");

    let html: String = std::fs::read_to_string("dash.html").expect("Unable to load dash.html").parse().expect("Unable to convert dash.html to a string");
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
        .expect("Failed to build_web_view");
    view.set_fullscreen(true);
    view
}

fn inject_user_configuration(source: &WebView<()>) {
    let user_configuration: String = std::fs::read_to_string("user_configuration.json").expect("Failed to read user_configuration.json").parse::<String>().expect("Failed to convert user_configuration.json to a string").replace("\n", "");
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

static mut LAST_UPDATE_RUN: SystemTime = SystemTime::UNIX_EPOCH;
fn update_loop(source: &WebView<()>) {
    let handle = source.handle();
    let mut odometer = Odometer::new();

    unsafe {
        LAST_UPDATE_RUN = SystemTime::now();
    }
    thread::spawn(move || {
        let provider: &mut dyn CanProvider = &mut PsuedoCanProvider { };
        odometer.auto_save();

        loop {
            let mut can_data = provider.can_data();

            let result = handle.dispatch(move |view| {
                let time_since_last_run: Duration;
                unsafe {
                    time_since_last_run = SystemTime::now().duration_since(LAST_UPDATE_RUN).expect("Whoops, time ran backwards");
                    LAST_UPDATE_RUN = SystemTime::now();
                }

                let odometer_reading = odometer.update(can_data.get("vss1").unwrap().parse().unwrap(), time_since_last_run);
                can_data.insert("odometer", odometer_reading.to_string());
                update_web_view(view, &can_data);
                Ok(())
            });

            if result.is_err() {
                error!("Failed to dispatch, {:?}", result.unwrap_err());
            }
        }
    });
}
