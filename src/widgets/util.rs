use std::collections::HashMap;
use std::cell::RefCell;
use anyhow::Result;

use dominator::Dom;

thread_local! {
    pub static WIDGET_REGISTRY: RefCell<HashMap<String,fn(&str)->Result<Box<dyn MovableWidget>>>>  = RefCell::new(HashMap::new());
}

pub trait MovableWidget {
    fn render( self: &Self ) -> Dom;
    fn serialize( self: &Self ) -> Result<String>;
    fn deserialize( data: &str ) -> Result<Box<dyn MovableWidget>> where Self: Sized;
}

pub fn register( name: String, typ: fn(&str) -> Result<Box<dyn MovableWidget>> ) {
    WIDGET_REGISTRY.with(|x| x.borrow_mut().insert(name, typ ));
}

pub fn get_type_deserializer( name: &str ) -> Option<fn(&str) -> Result<Box<dyn MovableWidget>>> {
    WIDGET_REGISTRY.with(|x| -> Option<fn(&str) -> Result<Box<dyn MovableWidget>>> {
        if x.borrow().contains_key( name ) {
            Some( x.borrow()[ name ] )
        } else { 
            None 
        }
    })
}