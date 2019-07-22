/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[macro_use]
extern crate mozjs;

use mozjs::jsapi::JSAutoRealm;
use mozjs::jsapi::JSContext;
use mozjs::jsapi::JS_DefineFunction;
use mozjs::jsapi::JS_NewGlobalObject;
use mozjs::jsapi::OnNewGlobalHookOption;
use mozjs::jsapi::Value;
use mozjs::jsval::UndefinedValue;
use mozjs::panic::wrap_panic;
use mozjs::rust::{JSEngine, RealmOptions, Runtime, SIMPLE_GLOBAL_CLASS};
use std::ptr;

#[test]
#[should_panic]
fn test_panic() {
    let engine = JSEngine::init().unwrap();
    let runtime = Runtime::new(engine);
    let context = runtime.cx();
    let h_option = OnNewGlobalHookOption::FireOnNewGlobalHook;
    let c_option = RealmOptions::default();

    unsafe {
        let global = JS_NewGlobalObject(context, &SIMPLE_GLOBAL_CLASS,
                                        ptr::null_mut(), h_option, &*c_option);
        rooted!(in(context) let global_root = global);
        let global = global_root.handle();
        let _ac = JSAutoRealm::new(context, global.get());
        let function = JS_DefineFunction(context, global.into(),
                                         b"test\0".as_ptr() as *const _,
                                         Some(test), 0, 0);
        assert!(!function.is_null());
        rooted!(in(context) let mut rval = UndefinedValue());
        let _ = runtime.evaluate_script(global, "test();", "test.js", 0,
                                        rval.handle_mut());
    }
}

unsafe extern "C" fn test(_cx: *mut JSContext, _argc: u32, _vp: *mut Value) -> bool {
    wrap_panic(|| {
        panic!()
    }, false)
}
