extern crate web_view;
extern crate rand;

#[macro_use]
extern crate log;
extern crate simple_logger;

use web_view::*;
use std::collections::HashMap;
use std::thread;
use std::time::Duration;
use rand::Rng;

// TODO: evaluate if this should be user_data or if update_loop
// can just get and pass the data without this context map.
// Main consideration is stale data and understanding if we will do
// "partial" updates or not.
type CanData = HashMap<&'static str, String>;

fn main() {
    simple_logger::init().unwrap();

    let html: String = std::fs::read_to_string("dash.html").unwrap().parse().unwrap();
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

fn update_loop(source: &WebView<CanData>) {
    let handle = source.handle();
    thread::spawn(move || loop {
        {
            let result = handle.dispatch(move |view| {
                // Mock data for PoC
                let rpm = rand::thread_rng().gen_range(0, 1000) + 6750;
                let rpm_string = format!("{}", rpm);
                let user_data = &mut *view.user_data_mut();
                user_data.insert("rpm", rpm_string);
                user_data.insert("vss", "10".to_string());
                user_data.insert("gear", "2".to_string());
                user_data.insert("fuel", "35".to_string());
                let update_cycle = view.eval(&format!("update({:?})", view.user_data()));
                if update_cycle.is_err() {
                    let error = update_cycle.unwrap_err();
                    error!("Failed to update view, {:?}", error);
                    Err(error)
                } else {
                    update_cycle
                }
            });

            if result.is_err() {
                error!("Failed to dispatch, {:?}", result.unwrap_err());
            }
        }
        thread::sleep(Duration::from_millis(200)); // PoC
    });
}
