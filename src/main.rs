use std::{time::Duration, rc::Rc};
use dominator::{Dom, html};
use gloo_console::log;

use serde_json::Value;
use widgets::{util::{MovableWidget, register, get_type_deserializer, Widget}, toast::{RcToast}, tabpanel::{TabPanel, TabPosition, TabPanelInfo}};

#[macro_use]
extern crate lazy_static;

mod utils;
pub mod widgets;

struct TestWidget;
struct TestWidget2;

impl Widget for TestWidget {
    fn render( &self ) -> Dom {
        html!("div", {
            .style("background-color", "red")
            .style("width", "100%")
            .style("height", "100%")
        })
    }
}

impl TabPanelInfo for TestWidget {
    fn title( &self ) -> String {
        "test title".to_owned()
    }
}

impl Widget for TestWidget2 {
    fn render( &self ) -> Dom {
        html!("div", {
            .style("background-color", "yellow")
            .style("width", "100%")
            .style("height", "100%")
        })
    }
}

impl TabPanelInfo for TestWidget2 {
    fn title( &self ) -> String {
        "test title".to_owned()
    }
}

thread_local! {
    pub static TABPANEL: Rc<TabPanel> = Rc::new(TabPanel::new());
}

fn main() {
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

    widgets::toast::TOAST_CONTAINER.with(|x| x.set_toast_position(widgets::toast::ToastPosition::TopRight));

    widgets::toast::render_toast_container();

    widgets::toast::Toast::info().title("This is a informational toast".to_owned()).text("Test".to_owned()).timeout(Duration::default()).open();
    widgets::toast::Toast::success().title("Success".to_owned()).text("You did it\nThis is a test for multiline text and also a test for how long a text the toast can handle".to_owned()).timeout(Duration::default()).open();
    widgets::toast::Toast::warning().title("Warning".to_owned()).text("You did it!".to_owned()).timeout(Duration::new(30,0)).open();
    let toast4 = widgets::toast::Toast::error();
    toast4.title("Error".to_owned()).text("You did it!".to_owned()).timeout(Duration::new(5,0));
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

    let widget1 = Rc::new(TestWidget{});

    TABPANEL.with(|x| x.add_tab(widget1, TabPosition::Start));

    let widget2 = Rc::new(TestWidget2{});

    TABPANEL.with(|x| x.add_tab(widget2, TabPosition::End));
    

    dominator::append_dom(&dominator::body(), TABPANEL.with(|x| x.render_with_mixin( &|dom| dom)));

}