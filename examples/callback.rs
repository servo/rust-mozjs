/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/. */

extern crate js;
extern crate libc;

use std::ffi::CStr;
use std::ptr;
use std::str;

use js::{JSCLASS_RESERVED_SLOTS_MASK,JSCLASS_RESERVED_SLOTS_SHIFT,JSCLASS_GLOBAL_SLOT_COUNT,JSCLASS_IS_GLOBAL};
use js::jsapi::JS_GlobalObjectTraceHook;
use js::jsapi::{CallArgs,CompartmentOptions,OnNewGlobalHookOption,Rooted,Value};
use js::jsapi::{JS_DefineFunction,JS_Init,JS_NewGlobalObject,JS_EncodeStringToUTF8,JS_ReportError};
use js::jsapi::{JSAutoCompartment,JSAutoRequest,JSContext,JSClass};
use js::jsval::UndefinedValue;
use js::rust::Runtime;

static CLASS: &'static JSClass = &JSClass {
    name: b"test\0" as *const u8 as *const libc::c_char,
    flags: JSCLASS_IS_GLOBAL | ((JSCLASS_GLOBAL_SLOT_COUNT & JSCLASS_RESERVED_SLOTS_MASK) << JSCLASS_RESERVED_SLOTS_SHIFT),
    addProperty: None,
    delProperty: None,
    getProperty: None,
    setProperty: None,
    enumerate: None,
    resolve: None,
    convert: None,
    finalize: None,
    call: None,
    hasInstance: None,
    construct: None,
    trace: Some(JS_GlobalObjectTraceHook),
    reserved: [0 as *mut _; 25]
};

fn main() {
    unsafe {
        JS_Init();
    }
    let runtime = Runtime::new();
    let context = runtime.cx();

    unsafe {
        let h_option = OnNewGlobalHookOption::FireOnNewGlobalHook;
        let c_option = CompartmentOptions::default();
        let _ar = JSAutoRequest::new(context);
        let global = JS_NewGlobalObject(context, CLASS, ptr::null_mut(), h_option, &c_option);
        let global_root = Rooted::new(context, global);
        let global = global_root.handle();
        let _ac = JSAutoCompartment::new(context, global.get());
        JS_DefineFunction(context, global, b"puts\0".as_ptr() as *const libc::c_char, Some(puts), 1, 0);
        let javascript = "puts('Test Iñtërnâtiônàlizætiøn ┬─┬ノ( º _ ºノ) ');".to_string();
        let _ = runtime.evaluate_script(global, javascript, "test.js".to_string(), 0);
    }
}

unsafe extern "C" fn puts(context: *mut JSContext, argc: u32, vp: *mut Value) -> bool {
    let args = CallArgs::from_vp(vp, argc);

    if args._base.argc_ != 1 {
        JS_ReportError(context, b"puts() requires exactly 1 argument\0".as_ptr() as *const libc::c_char);
        return false;
    }

    let arg = args.get(0);
    let js = js::rust::ToString(context, arg);
    let message_root = Rooted::new(context, js);
    let message = JS_EncodeStringToUTF8(context, message_root.handle());
    let message = CStr::from_ptr(message);
    println!("{}", str::from_utf8(message.to_bytes()).unwrap());

    args.rval().set(UndefinedValue());
    return true;
}
