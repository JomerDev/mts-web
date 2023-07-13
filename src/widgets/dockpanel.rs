use std::{cell::RefCell, rc::Rc};

use dominator::{html, Dom};
use futures_signals::signal::Mutable;
use gloo_timers::callback::Timeout;
use web_sys::{Element, EventTarget};

use super::{boxengine::{Sizer, BoxEngine}, tabpanel::{TabPanel, Tab}};


pub enum DockPanelNode {
    Split(SplitLayoutNode),
    Tab(TabLayoutNode),
    None
}

struct DockPanel {
    root: DockPanelNode,
    overlay: DockPanelOverlay
}

impl DockPanel {


    fn split_root( mut self, orientation: SplitOrientation ) {
        let (old_root, orientation_matches) = match self.root {
            DockPanelNode::Split( node ) => {
                if node.orientation == orientation {
                    (Some(node), true)
                } else {
                    (Some(node), false)
                }
            },
            _ => (None, false)
        };

        if old_root.is_none() || !orientation_matches {
            let mut new_root = SplitLayoutNode::new( orientation );

            if old_root.is_some() {
                new_root.children.push( DockPanelNode::Split(old_root.expect("Not possible")) )
            }
            self.root = DockPanelNode::Split(new_root);
        } else {
            self.root = DockPanelNode::Split(old_root.expect("Not possible"));
        }
    }
}


pub struct TabLayoutNode {
    tab_sizer: Sizer,
    widget_sizer: Sizer,
    tabbar: TabPanel
}

impl TabLayoutNode {

    pub fn new( tab: TabPanel ) -> Self {
        let mut tab_sizer = BoxEngine::create_sizer(None);
        tab_sizer.stretch = 0.0;
        let mut widget_sizer = BoxEngine::create_sizer(None);
        widget_sizer.stretch = 1.0;

        TabLayoutNode {
            tab_sizer,
            widget_sizer,
            tabbar: tab
        }
    }

    pub fn find_tab_node(&self, tab: &Rc<Tab>) -> Option<&Self> {
        self.tabbar.get_tab_index(tab).and(Some(&self))
    }

    pub fn find_first_tab_node(&self) -> &Self {
        self
    }

    pub fn find_split_node<'a>() -> Option<&'a SplitLayoutNode> {
        None
    }

    pub fn hit_tab_panel(&self, target: EventTarget ) -> bool {
        self.tabbar.check_if_over_tabpanel( target )
    }


}

#[derive(Debug, PartialEq, Eq)]
pub enum SplitOrientation {
    Horizontal,
    Vertical
}
pub struct SplitLayoutNode {
    normalized: bool,
    pub children: Vec<DockPanelNode>,
    pub sizers: Vec<Sizer>,
    handles: Vec<Element>,
    orientation: SplitOrientation
}

impl SplitLayoutNode {

    pub fn new( orientation: SplitOrientation ) -> Self {
        SplitLayoutNode {
            orientation,
            children: vec![],
            sizers: vec![],
            handles: vec![],
            normalized: false
        }
    }

}

struct DockPanelOverlay {
    timer: RefCell<Option<Timeout>>,
    hidden: Mutable<bool>,
    top: Mutable<String>,
    left: Mutable<String>,
    right: Mutable<String>,
    bottom: Mutable<String>,
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

    pub fn render(&self) -> Dom {
        html!("div", {
            .visible_signal( self.hidden.signal() )
            .class("mtw-dockpanel-overlay")
            .style_signal("top", self.top.signal_cloned())
            .style_signal("left", self.left.signal_cloned())
            .style_signal("right", self.right.signal_cloned())
            .style_signal("bottom", self.bottom.signal_cloned())
        })
    }

    pub fn show(self: &Rc<Self>, top: i32, left: i32, right: i32, bottom: i32) {
        self.top.set(format!("{}px", top));
        self.left.set(format!("{}px", left));
        self.right.set(format!("{}px", right));
        self.bottom.set(format!("{}px", bottom));

        if let Some(timer) = self.timer.borrow_mut().take() {
            timer.cancel();
        }

        self.hidden.set(false);
    }

    pub fn hide(self: &Rc<Self>, delay: u32) {
        if !self.hidden.get() {
            // Hide immediately if delay is 0
            if delay <= 0 {
                if let Some(timer) = self.timer.borrow_mut().take() {
                    timer.cancel();
                }
                self.hidden.set(true);
            }

            let this = self.clone();
            // Do nothing if a hide is already pending.
            if self.timer.borrow().is_none() {
                *self.timer.borrow_mut() = Some(Timeout::new(delay, move || {
                    *this.timer.borrow_mut() = None;
                    this.hidden.set(true);
                }));
            }
        }
    }
}
