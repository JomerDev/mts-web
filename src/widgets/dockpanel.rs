use std::{rc::Rc, cell::RefCell};

use dominator::{html, Dom};
use futures_signals::signal::Mutable;
use gloo_timers::callback::Timeout;








struct DockPanelOverlay {
    timer: RefCell<Option<Timeout>>,
    hidden: Mutable<bool>,
    top: Mutable<String>,
    left: Mutable<String>,
    right: Mutable<String>,
    bottom: Mutable<String>
}

impl DockPanelOverlay {

    pub fn new() -> Rc<Self> {
        Rc::new(Self {
            timer: RefCell::new(None),
            hidden: Mutable::new(true),
            top: Mutable::new("0".to_owned()),
            left: Mutable::new("0".to_owned()),
            right: Mutable::new("0".to_owned()),
            bottom: Mutable::new("0".to_owned()),
        })
    }

    pub fn render( &self ) -> Dom {
        html!("div", {
            .visible_signal( self.hidden.signal() )
            .class("mtw-dockpanel-overlay")
            .style_signal("top", self.top.signal_cloned())
            .style_signal("left", self.left.signal_cloned())
            .style_signal("right", self.right.signal_cloned())
            .style_signal("bottom", self.bottom.signal_cloned())
        })
    }

    pub fn show( self: &Rc<Self>, top: i32, left: i32, right: i32, bottom: i32 ) {

        self.top.set( format!("{}px", top) );
        self.left.set( format!("{}px", left) );
        self.right.set( format!("{}px", right) );
        self.bottom.set( format!("{}px", bottom) );

        if let Some( timer ) = self.timer.borrow_mut().take() {
            timer.cancel();
        }

        self.hidden.set(false);
    }

    pub fn hide( self: &Rc<Self>, delay: u32 ) {
        if !self.hidden.get() {
            // Hide immediately if delay is 0
            if delay <= 0 {
                if let Some( timer ) = self.timer.borrow_mut().take() {
                    timer.cancel();
                }
                self.hidden.set(true);
            }

            let this = self.clone();
            // Do nothing if a hide is already pending.
            if self.timer.borrow().is_none() {
                *self.timer.borrow_mut() = Some(Timeout::new( delay, move || {
                    *this.timer.borrow_mut() = None;
                    this.hidden.set(true);
                }));
            }
        }
    }

}