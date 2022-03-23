use std::time::Duration;
use gloo_console::log;

use serde_json::Value;
use widgets::{util::{MovableWidget, register, get_type_deserializer}, toast::{RcToast}};

mod utils;
pub mod widgets;


fn main() {
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

    widgets::toast::render_toast_container();

    widgets::toast::TOAST_CONTAINER.with(|x| x.set_toast_position(widgets::toast::ToastPosition::TopCenter));

    widgets::toast::Toast::info().title("This is a informational toast".to_owned()).text("Test".to_owned()).timeout(Duration::default()).open();
    widgets::toast::Toast::success().title("Success".to_owned()).text("You did it\nThis is a test for multiline text and also a test for how long a text the toast can handle".to_owned()).timeout(Duration::default()).open();
    widgets::toast::Toast::warning().title("Warning".to_owned()).text("You did it!".to_owned()).timeout(Duration::new(30,0)).open();
    let toast4 = widgets::toast::Toast::error();
    toast4.title("Error".to_owned()).text("You did it!".to_owned()).timeout(Duration::new(5,0));
    toast4.open();

    register("Toast".to_owned(), RcToast::deserialize );

    let str = toast4.serialize();
    let string = str.expect("Has error");
    log!(&string);

    let json: Value = serde_json::from_str(&string).expect("No json possible");
    let widget_type = json.get("widget_type").unwrap().as_str().unwrap();

    if let Some(deserializer) = get_type_deserializer(widget_type) {
        let widget = deserializer(&string).unwrap();
        dominator::append_dom(&dominator::body(), widget.render());
    }
    
}