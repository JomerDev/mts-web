use dominator::{html, Dom};
use gloo_console::log;
use std::{rc::Rc, time::Duration};

use serde_json::Value;
use widgets::{
    tabpanel::{TabPanel, TabPosition, Tab},
    toast::RcToast,
    util::{get_type_deserializer, register, MovableWidget, Widget},
};

#[macro_use]
extern crate lazy_static;

mod utils;
pub mod widgets;

struct TestWidget {
    pub color: String
}

impl Widget for TestWidget {
    fn render(&self) -> Dom {
        html!("div", {
            .style("background-color", &self.color)
            .style("width", "100%")
            .style("height", "100%")
        })
    }
}

thread_local! {
    pub static TABPANEL: Rc<TabPanel> = Rc::new(TabPanel::new());
}

fn main() {
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

    widgets::toast::TOAST_CONTAINER
        .with(|x| x.set_toast_position(widgets::toast::ToastPosition::TopRight));

    widgets::toast::render_toast_container();

    widgets::toast::Toast::info()
        .title("This is a informational toast".to_owned())
        .text("Test".to_owned())
        .timeout(Duration::default())
        .open();
    widgets::toast::Toast::success().title("Success".to_owned()).text("You did it\nThis is a test for multiline text and also a test for how long a text the toast can handle".to_owned()).timeout(Duration::default()).open();
    widgets::toast::Toast::warning()
        .title("Warning".to_owned())
        .text("You did it!".to_owned())
        .timeout(Duration::new(30, 0))
        .open();
    let toast4 = widgets::toast::Toast::error();
    toast4
        .title("Error".to_owned())
        .text("You did it!".to_owned())
        .timeout(Duration::new(5, 0));
    toast4.open();

    // register("Toast".to_owned(), RcToast::deserialize );

    // let str = toast4.serialize();
    // let string = str.expect("Has error");
    // log!(&string);

    // let json: Value = serde_json::from_str(&string).expect("No json possible");
    // let widget_type = json.get("widget_type").unwrap().as_str().unwrap();

    // if let Some(deserializer) = get_type_deserializer(widget_type) {
    // let widget = deserializer(&string).unwrap();
    // dominator::append_dom(&dominator::body(), widget.render());
    // }

    let widget1 = TestWidget { color: String::from("red") };

    let mut tab1 = Tab::new();
    tab1.set_title(String::from("Tab 1")).set_closable(true);
    // tab1.set_render_content(Box::new(move || widget1.render()));

    TABPANEL.with(|x| x.add_tab(Rc::new(tab1), TabPosition::Start));

    let widget2 = TestWidget { color: String::from("green") };

    let mut tab2 = Tab::new();
    tab2.set_title(String::from("Tab 2")).set_closable(true);
    // tab2.set_render_content(Box::new(move || widget2.render()));

    TABPANEL.with(|x| x.add_tab(Rc::new(tab2), TabPosition::End));

    let widget3 = TestWidget { color: String::from("yellow") };

    let mut tab3 = Tab::new();
    tab3.set_title(String::from("Tab 3")).set_closable(true);
    // tab3.set_render_content(Box::new(move || widget3.render()));

    TABPANEL.with(|x| x.add_tab(Rc::new(tab3), TabPosition::End));

    dominator::append_dom(
        &dominator::body(),
        TABPANEL.with(|x| x.render_with_mixin(&|dom| dom)),
    );
}
