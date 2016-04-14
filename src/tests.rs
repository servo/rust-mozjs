
use consts::{JSCLASS_RESERVED_SLOTS_MASK, JSCLASS_GLOBAL_SLOT_COUNT, JSCLASS_IS_GLOBAL};
use jsapi::{JSCLASS_RESERVED_SLOTS_SHIFT, JS_GlobalObjectTraceHook};
use conversions::*;
use jsval::*;
use rust::*;
use jsapi::{CallArgs,CompartmentOptions,OnNewGlobalHookOption,Rooted,Value, JS_NewGlobalObject};
use jsapi::{RootedValue, RootedObject, JSAutoRequest, JSAutoCompartment, JSClass};
use std::ptr;

static CLASS: &'static JSClass = &JSClass {
    name: b"test\0" as *const u8 as *const _,
    flags: JSCLASS_IS_GLOBAL | ((JSCLASS_GLOBAL_SLOT_COUNT & JSCLASS_RESERVED_SLOTS_MASK) << JSCLASS_RESERVED_SLOTS_SHIFT),
    addProperty: None,
    delProperty: None,
    getProperty: None,
    setProperty: None,
    enumerate: None,
    resolve: None,
    mayResolve: None,
    finalize: None,
    call: None,
    hasInstance: None,
    construct: None,
    trace: Some(JS_GlobalObjectTraceHook),
    reserved: [0 as *mut _; 23]
};


#[test]
fn test_vec_conversion() {
    let rt = Runtime::new();
    let cx = rt.cx();

    let glob = RootedObject::new(cx, 0 as *mut _);
    let _ar = JSAutoRequest::new(cx);

    let h_option = OnNewGlobalHookOption::FireOnNewGlobalHook;
    let c_option = CompartmentOptions::default();
    let global = unsafe {
        JS_NewGlobalObject(cx, CLASS, ptr::null_mut(), h_option, &c_option)
    };
    let global_root = Rooted::new(cx, global);
    let global = global_root.handle();

    let _ac = JSAutoCompartment::new(cx, global.get());

    let mut rval = RootedValue::new(cx, UndefinedValue());

    let orig_vec: Vec<f32> = vec![1.0, 2.9, 3.0];
    let converted = unsafe {
        orig_vec.to_jsval(cx, rval.handle_mut());
        Vec::<f32>::from_jsval(cx, rval.handle(), ()).unwrap()
    };

    assert_eq!(orig_vec, converted);

    let orig_vec: Vec<i32> = vec![1, 2, 3];
    let converted = unsafe {
        orig_vec.to_jsval(cx, rval.handle_mut());
        Vec::<i32>::from_jsval(cx, rval.handle(), ConversionBehavior::Default).unwrap()
    };

    assert_eq!(orig_vec, converted);
}

#[test]
fn stack_limit() {
    let rt = Runtime::new();
    let cx = rt.cx();
    let _ar = JSAutoRequest::new(cx);

    let h_option = OnNewGlobalHookOption::FireOnNewGlobalHook;
    let c_option = CompartmentOptions::default();
    let global = unsafe {
        JS_NewGlobalObject(cx, CLASS, ptr::null_mut(), h_option, &c_option)
    };
    let global_root = Rooted::new(cx, global);
    let global = global_root.handle();

    assert!(rt.evaluate_script(global,
                               "function f() { f.apply() } f()".to_string(),
                               "test".to_string(),
                               1).is_err());
}

