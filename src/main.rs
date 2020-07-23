extern crate web_view;
extern crate rand;

use web_view::*;
use std::collections::HashMap;
use std::thread;
use std::time::Duration;
use rand::Rng;

fn main() {
    let mut can_data: HashMap<&str, String> = HashMap::new();

    let mut view = web_view::builder()
        .title("Rust Can Dash")
        .content(Content::Html(HTML))
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

    let handle = view.handle();
    thread::spawn(move || loop {
        {
            handle.dispatch(move |mut view| {
                let rand_rpm: u32 = rand::thread_rng().gen_range(0, 50) + 850;
                let rpm_string = format!("{}", rand_rpm);
                (*view.user_data_mut()).insert("rpm", rpm_string);
                render(&mut view)
            });
        }
        thread::sleep(Duration::from_millis(200));
    });
    view.run();
}

fn render(view: &mut WebView<HashMap<&str, String>>) -> WVResult {
    let user_data = view.user_data();
    view.eval(&format!("update({:?})", user_data))
}

const HTML: &str = r#"
<!doctype html>
<html>
    <body>
        <strong>RPM: <span id="rpm"></span></strong>
        <br>
        <button onclick="external.invoke('enter')">enter fullscreen</button>
        <button onclick="external.invoke('exit')">exit fullscreen</button>

        <script type="text/javascript">
            function update(data) {
                document.getElementById('rpm').innerHTML = data.rpm;
            }
        </script>
	</body>
</html>
"#;
