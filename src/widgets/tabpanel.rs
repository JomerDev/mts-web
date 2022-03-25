use std::{cmp, rc::Rc};

use dominator::{Dom, html, DomBuilder, clone, events};
use futures_signals::{signal::{Mutable, SignalExt}, signal_vec::{MutableVec, SignalVecExt}};
use web_sys::HtmlElement;

use super::util::Widget;

pub trait TabPanelInfo: Widget {
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
    tabs: MutableVec<Rc<dyn TabPanelInfo>>
}

impl TabPanel {
    pub fn new() -> Self {
        TabPanel {
            current_tab: Mutable::new(0),
            tabs: MutableVec::new()
        }
    }

    pub fn render_with_mixin( self: &Rc<TabPanel>, mixin: &dyn Fn(DomBuilder<HtmlElement>) -> DomBuilder<HtmlElement> ) -> Dom {
        let panel = self.clone();
        html!("div", {
            .style("width", "1000px")
            .style("height", "700px")
            .apply(mixin)
            .class("mtw-tabpanel")
            .children(&mut [
                html!("ul", {
                    .class("mtw-tabbar")
                    .children_signal_vec(self.tabs.signal_vec_cloned()
                        .enumerate().map(clone!( panel => move |(index, widget)| {
                            let index = index.lock_ref().unwrap();
                            html!("li", {
                            .class("mtw-tabbar-tab")
                            .children(&mut [
                                html!("span", {
                                    .class("mtw-tab-icon")
                                }),
                                html!("span", {
                                    .class("mtw-tab-title")
                                    .text( &widget.as_ref().title() )
                                }),
                                html!("span", {
                                    .class("mtw-tab-close")
                                    .class("mtw-tab-closable")
                                    .text("×")
                                    .event(clone!(panel => move |_: events::Click| {
                                        panel.remove_tab( index );
                                    }))
                                })
                            ])
                            .class_signal("mtw-tab-active", panel.current_tab.signal().map( move |current_tab| current_tab == index ) )
                            .event(clone!(panel => move |_: events::Click| {
                                panel.select_tab( index )
                            }))
                        })})))
                }),
                html!("div", {
                    .class("mtw-tab-container")
                    .children_signal_vec(self.tabs.signal_vec_cloned()
                        .enumerate().map(clone!( panel => move |(index, widget)| 
                            html!("div", {
                                .class("mtw-tab-frame")
                                .child( widget.render() )
                                .visible_signal( panel.current_tab.signal().map( move |current_tab| current_tab == index.lock_ref().unwrap() ) )
                            })
                        )))
                }),
            ])
        })
    }

    pub fn add_tab( &self, widget: Rc<dyn TabPanelInfo>, position: TabPosition ) {
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

    pub fn remove_tab( &self, index: usize ) {
        let len = self.tabs.lock_ref().len();
        let index = cmp::min(index, len - 1);
        self.tabs.lock_mut().remove(index);
        let len = self.tabs.lock_ref().len();
        if self.current_tab.get() >= len - 1 {
            self.current_tab.set(len - 1);
        }
    }
}