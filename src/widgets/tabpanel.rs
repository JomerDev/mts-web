use std::{cmp, rc::Rc, str::FromStr};

use dominator::{clone, events, html, Dom, DomBuilder};
use futures_signals::{
    signal::{Mutable, SignalExt, ReadOnlyMutable, Signal},
    signal_vec::{MutableVec, SignalVecExt}, map_ref,
};
use wasm_bindgen::JsCast;
use web_sys::{HtmlElement, EventTarget};
use uuid::Uuid;

fn builder_noop(builder: DomBuilder<HtmlElement>) -> DomBuilder<HtmlElement> {
    builder
}

pub struct Tab{
    title: Mutable<String>,
    closable: Mutable<bool>,
    icon: Mutable<Option<String>>,
    id: Uuid,
}

impl Tab{
    pub fn new() -> Self {
        Self {
            title: Mutable::new(String::from("")),
            closable: Mutable::new(true),
            icon: Mutable::new(None),
            id: Uuid::new_v4(),
        }
    }

    pub fn set_title(&self, title: String) -> &Self{
        self.title.set(title);
        self
    }

    pub fn set_closable(&self, closable: bool) -> &Self{
        self.closable.set(closable);
        self
    }

    pub fn set_icon(&self, icon: Option<String>) -> &Self {
        self.icon.set(icon);
        self
    }

    fn title(&self) -> &Mutable<String> {
        &self.title
    }

    fn render(&self) -> Dom {
        html!("div")
    }

    fn closable(&self) -> &Mutable<bool> {
        &self.closable
    }
}

pub enum TabPosition {
    Start,
    End,
    Index(usize),
}

pub struct TabPanel {
    current_tab: Mutable<usize>,
    tabs: MutableVec<Rc<Tab>>,
    id: Uuid,
}

impl TabPanel {
    pub fn new() -> Self {
        TabPanel {
            current_tab: Mutable::new(0),
            tabs: MutableVec::new(),
            id: Uuid::new_v4()
        }
    }

    fn is_currently_selected_tab(&self, index: &ReadOnlyMutable<Option<usize>>) -> impl Signal<Item = bool> {
        (map_ref! {
            let current_tab = self.current_tab.signal(),
            let index = index.signal() =>
            if let Some(idx) = index {
                current_tab == idx
            } else {
                false
            }
        })
        .dedupe()
    }


    pub fn render_with_mixin(
        self: &Rc<TabPanel>,
        mixin: &dyn Fn(DomBuilder<HtmlElement>) -> DomBuilder<HtmlElement>,
    ) -> Dom {
        let panel = self.clone();
        html!("div", {
            .attr("data-panel-id", &self.id.to_string())
            .style("width", "1000px")
            .style("height", "700px")
            .apply(mixin)
            .class("mtw-tabpanel")
            .children(&mut [
                html!("ul", {
                    .class("mtw-tabbar")
                    .children_signal_vec(self.tabs.signal_vec_cloned()
                    .enumerate().map(clone!( panel => move |(index, widget)| {
                        html!("li", {
                        .attr("data-tab-id", &widget.id.to_string())
                        .class("mtw-tabbar-tab")
                        .children(&mut [
                            html!("span", {
                                .class("mtw-tab-icon")
                            }),
                            html!("span", {
                                .class("mtw-tab-title")
                                .text_signal( widget.title().signal_cloned() )
                            }),
                            html!("span", {
                                .visible_signal(widget.closable().signal())
                                .class("mtw-tab-close")
                                .class("mtw-tab-closable")
                                .text("×")
                                .event(clone!(panel, index => move |_: events::Click| {
                                    panel.remove_tab( index.get().unwrap() );
                                }))
                            })
                        ])
                        .class_signal("mtw-tab-active", panel.is_currently_selected_tab( &index ) )
                        .event(clone!(panel, index => move |_: events::Click| {
                            panel.select_tab( index.get().unwrap() )
                        }))
                        .event(clone!(panel => move |event: events::DragStart| {

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
                                .visible_signal( panel.is_currently_selected_tab( &index ) )
                            })
                        )))
                }),
            ])
        })
    }

    pub fn add_tab(&self, widget: Rc<Tab>, position: TabPosition) {
        match position {
            TabPosition::Index(i) => {
                let i = cmp::min(i, self.tabs.lock_ref().len() - 1);
                self.tabs.lock_mut().insert_cloned(i, widget);
            }
            TabPosition::End => self.tabs.lock_mut().push_cloned(widget),
            TabPosition::Start => self.tabs.lock_mut().insert_cloned(0, widget),
        }
    }

    pub fn get_tab_index(&self, tab: &Rc<Tab> ) -> Option<usize> {
        self.tabs.lock_ref().into_iter().position(|r| Rc::ptr_eq(r, tab))
    }

    pub fn select_tab(&self, index: usize) {
        let len = self.tabs.lock_ref().len();
        let index = cmp::min(index, len - 1);
        self.current_tab.set(index);
    }

    pub fn remove_tab(&self, index: usize) {
        let len = self.tabs.lock_ref().len();
        let index = cmp::min(index, len - 1);
        self.tabs.lock_mut().remove(index);
        let len: usize = self.tabs.lock_ref().len();
        if self.current_tab.get() >= len - 1 {
            self.current_tab.set(len - 1);
        }
    }

    pub fn check_if_over_tabpanel(&self, target: &EventTarget ) -> bool {
        if let Some(element) = target.dyn_ref::<web_sys::Element>() {
            element.closest(&format!("div[data-panel-id=\"{}\"", self.id.to_string())).is_ok_and(|e| e.is_some())
        } else {
            false
        }
    }

    pub fn get_tab_index_from_target(&self, target: &EventTarget ) -> Option<usize> {
        if self.check_if_over_tabpanel(target) {
            target.dyn_ref::<web_sys::Element>()
                .and_then(|ele| ele.closest(".mtw-tabbar-tab").unwrap_or(None))
                .and_then(|t| t.get_attribute("data-tab-id"))
                .and_then(|s| Uuid::from_str(&s).ok())
                .and_then(|id| self.tabs.lock_ref().into_iter().position(|t| t.id == id))
        } else {
            None
        }
    }
}
