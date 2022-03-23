use std::{time::Duration, rc::Rc, cell::{RefCell}};
use dominator::{Dom, html, events, clone};
use dominator::{with_node, RefFn, DomBuilder};
use dominator::animation::{MutableAnimation, Percentage};
use dominator::traits::*;
use futures_signals::{signal::{Mutable, Signal, SignalExt}, signal_vec::{MutableVec, SignalVecExt}, map_ref};
use gloo_timers::callback::Timeout;


thread_local! {
    pub static TOAST_CONTAINER: Rc<ToastContainer> = ToastContainer::new();
}

#[derive(Debug, Copy, Clone)]
pub enum ToastPosition {
    TopLeft,
    TopCenter,
    TopRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

pub enum ToastType {
    Info,
    Warning,
    Error,
    Success
}

pub struct Toast {
    title: Mutable<String>,
    typ: ToastType,
    text: Mutable<String>,
    has_close_button: Mutable<bool>,
    close_on_click: Mutable<bool>,
    timeout: Mutable<Duration>,
    has_progress_bar: Mutable<bool>,
    closed: Mutable<bool>,
    id: u32,
    timeout_id: RefCell<Option<Timeout>>,
    timeout_paused: Mutable<bool>,
}

impl Toast {

    pub fn new( typ: ToastType ) -> Rc<Self> {
        Rc::new(Self {
            typ,
            title: Mutable::new("".to_owned()),
            text: Mutable::new("".to_owned()),
            has_close_button: Mutable::new(false),
            close_on_click: Mutable::new(true),
            timeout: Mutable::new(Duration::new(5,0)),
            has_progress_bar: Mutable::new(true),
            closed: Mutable::new(true),
            id: TOAST_CONTAINER.with(|x| x.get_new_id()),
            timeout_id: RefCell::new(None),
            timeout_paused: Mutable::new(false)
        })
    }

    pub fn timeout<'a>(self: &'a Rc<Toast>, time: Duration) -> &'a Rc<Toast> {
        self.timeout.set(time);
        if time == Duration::default() {
            self.has_progress_bar.set(false);
        }
        self
    }

    pub fn title<'a>(self: &'a Rc<Toast>, title: String) -> &'a Rc<Toast> {
        self.title.set(title);
        self
    }

    pub fn text<'a>(self: &'a Rc<Toast>, text: String) -> &'a Rc<Toast> {
        self.text.set(text);
        self
    }

    pub fn set_close_button_visible<'a>(self: &'a Rc<Toast>, visible: bool) -> &'a Rc<Toast> {
        self.has_close_button.set(visible);
        self
    }

    pub fn set_close_on_click<'a>(self: &'a Rc<Toast>, close: bool) -> &'a Rc<Toast> {
        self.close_on_click.set(close);
        self
    }

    pub fn set_progress_bar_visible<'a>(self: &'a Rc<Toast>, visible: bool) -> &'a Rc<Toast> {
        self.has_progress_bar.set(visible);
        self
    }

    pub fn info( ) -> Rc<Self> {
        Self::new( ToastType::Info )
    }

    pub fn warning( ) -> Rc<Self> {
        Self::new( ToastType::Warning )
    }

    pub fn error( ) -> Rc<Self> {
        Self::new( ToastType::Error )
    }

    pub fn success( ) -> Rc<Self> {
        Self::new( ToastType::Success )
    }

    fn type_to_class(&self) -> String {
        match self.typ {
            ToastType::Warning => "mtw-toast-warning".to_owned(),
            ToastType::Info => "mtw-toast-info".to_owned(),
            ToastType::Error => "mtw-toast-error".to_owned(),
            ToastType::Success => "mtw-toast-success".to_owned(),
        }
    }

    fn show_progress_bar(&self) -> impl Signal<Item = bool> {
        (map_ref! {
            let has_progress_bar = self.has_progress_bar.signal(),
            let timeout_paused = self.timeout_paused.signal() =>
            *has_progress_bar && !*timeout_paused
        }).dedupe()
    }

    pub fn render( self: Rc<Toast> ) -> Dom {
        let toast = &self;

        html!("div", {
            .class("mtw-toast")
            .class(self.type_to_class())
            .class_signal("mtw-toast-fadeout", self.closed.signal())
            .class_signal("mtw-toast-click-close", self.close_on_click.signal())
            .apply_if( self.timeout.get() != Duration::default(), |dom| dom
                .style("animation-delay", format!("{}ms",self.timeout.get().as_millis()) )
                .class("")
            )
            .event(clone!(toast => move |_: events::Click| {
                if toast.close_on_click.get() {
                    toast.close();
                }
            }))
            .event(clone!(toast => move |_: events::MouseMove| {
                toast.stop_timeout();
                toast.timeout_paused.set(true);
            }))
            .event(clone!(toast => move |_: events::MouseLeave| {
                toast.start_timeout();
                toast.timeout_paused.set(false);
            }))
            .children(&mut [
                html!("button", {
                    .class("mtw-toast-close-button")
                    .text("×")
                    .visible_signal(self.has_close_button.signal())
                    .event(clone!(toast => move |_: events::Click| {
                        toast.close();
                    }))
                }),
                html!("div", {
                    .class("mtw-toast-title")
                    .text_signal(self.title.signal_cloned())
                }),
                html!("div", {
                    .class("mtw-toast-text")
                    .text_signal(self.text.signal_cloned())
                }),
                html!("div", {
                    .class("mtw-toast-progress-bar")
                    .visible_signal(self.show_progress_bar())
                    .style("animation-duration", format!("{}ms",self.timeout.get().as_millis() + 1000) )
                })
            ])
        })
    }

    fn start_timeout( self: &Rc<Toast> ) {
        if self.timeout.get() != Duration::default() {
            let toast = self.clone();
        
            let id = Timeout::new(self.timeout.get().as_millis() as u32, move || {
                toast.close();
            });
            *self.timeout_id.borrow_mut() = Some(id);
        }
    }

    fn stop_timeout( self: &Rc<Toast> ) {
        if let Some(timout_id) = self.timeout_id.borrow_mut().take() {
            timout_id.cancel();
        }
    }

    pub fn open<'a>( self: &'a Rc<Toast> ) -> &'a Rc<Toast> {
        if self.closed.get() {
            self.closed.set(false);
            TOAST_CONTAINER.with(|x| x.add_toast( self.clone() ));
            if self.timeout.get() != Duration::default() {
                self.start_timeout();
            }
        }
        self
    }

    pub fn close( self: &Rc<Toast> ) {
        if !self.closed.get() {
            self.closed.set(true);
            let toast = self.clone();
            Timeout::new(1_000, move || {
                TOAST_CONTAINER.with(|x| x.remove_toast( toast ));
            }).forget();
        }
    }
}

impl PartialEq<Toast> for Toast {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}


pub struct ToastContainer {
    toasts: MutableVec<Rc<Toast>>,
    position: Mutable<ToastPosition>,
    toast_id: Mutable<u32>
}

impl ToastContainer {

    fn new() -> Rc<Self> {
        Rc::new(Self {
            toasts: MutableVec::new(),
            position: Mutable::new(ToastPosition::TopCenter),
            toast_id: Mutable::new(0)
        })
    }

    fn has_toasts( &self ) -> impl Signal<Item = bool> {
        self.toasts.signal_vec_cloned()
            .len()
            .map(|len| len != 0)
            .dedupe()
    }

    fn position_to_class(&self) -> String {
        match self.position.get() {
            ToastPosition::TopCenter => "mtw-toast-top-center".to_owned(),
            ToastPosition::TopLeft => "mtw-toast-top-left".to_owned(),
            ToastPosition::TopRight => "mtw-toast-top-right".to_owned(),
            ToastPosition::BottomLeft => "mtw-toast-bottom-left".to_owned(),
            ToastPosition::BottomCenter => "mtw-toast-bottom-center".to_owned(),
            ToastPosition::BottomRight => "mtw-toast-bottom-right".to_owned(),
        }
    }

    fn render( &self ) -> Dom {
        html!("div", {
            .class("mtw-toast-container")
            .class(self.position_to_class())
            .visible_signal( self.has_toasts() )
            .children_signal_vec(self.toasts.signal_vec_cloned()
                .map(|toast| toast.render( )))
        })
    }

    fn add_toast( &self, toast: Rc<Toast> ) {
        self.toasts.lock_mut().push_cloned(toast);
    }

    fn remove_toast( &self, toast: Rc<Toast> ) {
        self.toasts.lock_mut().retain(|x| **x != *toast);
    }

    fn get_new_id(&self) -> u32 {
        self.toast_id.set(self.toast_id.get() + 1);
        self.toast_id.get()
    }

    pub fn set_toast_position(&self, position: ToastPosition) {
        self.position.set(position);
    }
}

pub fn render_toast_container() {
    dominator::append_dom(&dominator::body(), TOAST_CONTAINER.with(|x| x.render()));
}