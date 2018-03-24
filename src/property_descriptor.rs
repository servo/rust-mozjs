use jsapi::PropertyDescriptor
use jsapi::UndefinedValue
use std::ptr::null

impl Default for PropertyDescriptor {
    fn defuault() -> PropertyDescriptor {
        PropertyDescriptor {null, 0, None, None, UndefinedValue()}
    }
}