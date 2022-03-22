use std::time::Duration;

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
    widgets::toast::Toast::error().title("Error".to_owned()).text("You did it!".to_owned()).timeout(Duration::new(5,0)).open();
}