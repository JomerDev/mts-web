use std::{cmp, rc::Rc};

use dominator::{clone, events, html, Dom, DomBuilder, with_node};
use futures_signals::{
    signal::{Mutable, SignalExt},
    signal_vec::{MutableVec, SignalVecExt},
};
use gloo_console::log;
use lazy_static::__Deref;
use web_sys::{HtmlElement, Element};

use super::util::Widget;

fn builder_noop(builder: DomBuilder<HtmlElement>) -> DomBuilder<HtmlElement> {
    builder
}

pub struct Tab{
    title: Mutable<String>,
    closable: Mutable<bool>,
    icon: Mutable<Option<String>>,
    render_mixin: fn(DomBuilder<HtmlElement>) -> DomBuilder<HtmlElement>,
    render_content: Box<dyn Fn() -> Dom>,
}

impl Tab{
    pub fn new() -> Self {
        Self {
            title: Mutable::new(String::from("")),
            closable: Mutable::new(true),
            icon: Mutable::new(None),
            render_mixin: builder_noop,
            render_content: Box::new(|| html!("div"))
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

    pub fn set_render_mixin(&mut self, mixin: fn(DomBuilder<HtmlElement>) -> DomBuilder<HtmlElement>) -> &Self {
        self.render_mixin = mixin;
        self
    }

    pub fn set_render_content(&mut self, render: Box<dyn Fn() -> Dom>) -> &Self {
        self.render_content = render;
        self
    }

    fn title(&self) -> Mutable<String> {
        self.title.clone()
    }

    fn get_render_mixin(&self) -> fn(DomBuilder<HtmlElement>) -> DomBuilder<HtmlElement> {
        self.render_mixin
    }

    fn render(&self) -> Dom {
        (self.render_content)()
    }

    fn closable(&self) -> &Mutable<bool> {
        &self.closable
    }
}

fn get_index( node: &Element ) -> usize {
    let mut index = 0;
    let mut nod = node;
    while let Some(node) = nod.previous_element_sibling() {
        nod = &node;
        stack.push(&&node);
        index += 1;
    }
    index
}

pub trait TabPanelInfo: Widget {
    fn title(&self) -> String {
        "".to_owned()
    }
    fn closable(&self) -> bool {
        true
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
}

impl TabPanel {
    pub fn new() -> Self {
        TabPanel {
            current_tab: Mutable::new(0),
            tabs: MutableVec::new(),
        }
    }

    pub fn render_with_mixin(
        self: &Rc<TabPanel>,
        mixin: &dyn Fn(DomBuilder<HtmlElement>) -> DomBuilder<HtmlElement>,
    ) -> Dom {
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
                        log!("draw_index", index);
                        html!("li", {
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
                                .event(clone!(panel => move |_: events::Click| {
                                    panel.remove_tab( index );
                                }))
                            })
                        ])
                        .with_node!( node => {
                            .class_signal("mtw-tab-active", panel.current_tab.signal().map(|current_tab| current_tab == get_index(node.deref()) ) )
                        })
                        .event(clone!(panel => move |_: events::Click| {
                            panel.select_tab( index )
                        }))
                        .event(clone!(panel => move |event: events::DragStart| {

                        }))
                        .apply( widget.get_render_mixin() )
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

    pub fn select_tab(&self, index: usize) {
        let len = self.tabs.lock_ref().len();
        let index = cmp::min(index, len - 1);
        self.current_tab.set(index);
    }

    pub fn remove_tab(&self, index: usize) {
        let len = self.tabs.lock_ref().len();
        let index = cmp::min(index, len - 1);
        self.tabs.lock_mut().remove(index);
        let len = self.tabs.lock_ref().len();
        log!(self.current_tab.get(), len - 1);
        if self.current_tab.get() >= len - 1 {
            log!("set");
            self.current_tab.set(len - 1);
        }
    }
}
