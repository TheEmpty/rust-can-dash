extern crate web_view;
extern crate rand;

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
    let html: String = std::fs::read_to_string("dash.html").unwrap().parse().unwrap();
    let mut view = build_web_view(html);
    update_loop(&mut view);
    view.run();
}

fn build_web_view(html: String) -> WebView<'static, CanData> {
    let mut can_data: CanData = HashMap::new();
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
    return view;
}

fn update_loop(view: &mut WebView<CanData>) {
    let handle = view.handle();
    thread::spawn(move || loop {
        {
            handle.dispatch(move |mut view| {
                let rand_rpm: u32 = rand::thread_rng().gen_range(0, 50) + 850; // PoC
                let rpm_string = format!("{}", rand_rpm);
                (*view.user_data_mut()).insert("rpm", rpm_string);
                view.eval(&format!("update({:?})", view.user_data()))
            });
        }
        thread::sleep(Duration::from_millis(200)); // PoC
    });
}
