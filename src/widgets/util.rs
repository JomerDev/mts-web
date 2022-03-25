use std::collections::HashMap;
use std::cell::RefCell;
use anyhow::Result;

use dominator::{Dom, DomBuilder};
use web_sys::HtmlElement;

thread_local! {
    pub static WIDGET_REGISTRY: RefCell<HashMap<String,fn(&str)->Result<Box<dyn Widget>>>>  = RefCell::new(HashMap::new());
}

pub trait MovableWidget: Widget {
    fn serialize( self: &Self ) -> Result<String>;
    fn deserialize( data: &str ) -> Result<Box<dyn Widget>>;
}

pub fn register( name: String, typ: fn(&str) -> Result<Box<dyn Widget>> ) {
    WIDGET_REGISTRY.with(|x| x.borrow_mut().insert(name, typ ));
}

pub fn get_type_deserializer( name: &str ) -> Option<fn(&str) -> Result<Box<dyn Widget>>> {
    WIDGET_REGISTRY.with(|x| -> Option<fn(&str) -> Result<Box<dyn Widget>>> {
        if x.borrow().contains_key( name ) {
            Some( x.borrow()[ name ] )
        } else { 
            None 
        }
    })
}

fn builder_noop(builder: DomBuilder<HtmlElement>) -> DomBuilder<HtmlElement> {
    builder
}

pub trait Widget {
    fn render( self: &Self ) -> Dom {
        self.render_with_mixin(&builder_noop)
    }

    fn render_with_mixin( self: &Self, _f: &dyn Fn(DomBuilder<HtmlElement>) -> DomBuilder<HtmlElement> ) -> Dom {
        self.render()
    }
}
