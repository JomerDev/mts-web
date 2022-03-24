use std::{cmp, rc::Rc};

use dominator::{Dom, html};
use futures_signals::{signal::{Mutable, Signal, SignalExt}, signal_vec::{MutableVec, SignalVecExt}, map_ref};

use super::util::Widget;

pub trait TabPanelInfo {
    fn title( &self ) -> String {
        "".to_owned()
    }
    fn closable( &self ) -> bool {
        true
    }
}

impl<T> TabPanelInfo for &T
where
    T: Widget,
    T: ?Sized
{
    fn title( &self ) -> String {
        "".to_owned()
    }
    fn closable( &self ) -> bool {
        true
    }
}

pub enum TabPosition {
    Start,
    End,
    Index(usize)
}

pub struct TabPanel {
    current_tab: Mutable<usize>,
    tabs: MutableVec<Rc<dyn Widget>>
}

impl TabPanel {
    pub fn new() -> Self {
        TabPanel {
            current_tab: Mutable::new(0),
            tabs: MutableVec::new()
        }
    }

    pub fn render( &'static self ) -> Dom {
        html!("div", {
            .class("mtw-tabpanel")
            .children(&mut [
                html!("ul", {
                    .class("mtw-tabbar")
                    .children_signal_vec(self.tabs.signal_vec_cloned()
                        .enumerate().map(move |(index, widget)|  html!("li", {
                            .children(&mut [
                                html!("span", {
                                    .class("mtw-tab-title")
                                    .text( &widget.as_ref().title() )
                                })
                            ])
                            .class_signal("mtw-tab-active", self.current_tab.signal().map( move |current_tab| current_tab == index.lock_ref().unwrap() ) )
                        })))
                }),
                html!("div", {
                    .children_signal_vec(self.tabs.signal_vec_cloned()
                        .enumerate().map(move |(index, widget)| 
                            widget.render_with_mixin(&|dom| dom
                                .class("test")
                            ) 
                        ))
                }),
            ])
        })
    }

    pub fn add_tab( &self, widget: Rc<dyn Widget>, position: TabPosition ) {
        match position {
            TabPosition::Index(i) => {
                let i = cmp::min(i, self.tabs.lock_ref().len() - 1);
                self.tabs.lock_mut().insert_cloned(i, widget);
            },
            TabPosition::End => self.tabs.lock_mut().push_cloned(widget),
            TabPosition::Start => self.tabs.lock_mut().insert_cloned(0, widget),
        }
    }

    pub fn select_tab( &self, index: usize ) {
        let len = self.tabs.lock_ref().len();
        let index = cmp::min(index, len - 1);
        self.current_tab.set(index);
    }
}