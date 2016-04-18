/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

extern crate js;

use js::conversions::ConversionBehavior;
use js::conversions::FromJSValConvertible;
use js::conversions::ToJSValConvertible;
use js::jsapi::CompartmentOptions;
use js::jsapi::JSAutoCompartment;
use js::jsapi::JSAutoRequest;
use js::jsapi::JS_Init;
use js::jsapi::JS_NewGlobalObject;
use js::jsapi::OnNewGlobalHookOption;
use js::jsapi::Rooted;
use js::jsapi::RootedValue;
use js::jsval::UndefinedValue;
use js::rust::{Runtime, SIMPLE_GLOBAL_CLASS};

use std::ptr;

#[test]
fn vec_conversion() {
    unsafe {
        assert!(JS_Init());

        let rt = Runtime::new();
        let cx = rt.cx();
        let _ar = JSAutoRequest::new(cx);

        let h_option = OnNewGlobalHookOption::FireOnNewGlobalHook;
        let c_option = CompartmentOptions::default();
        let global = JS_NewGlobalObject(cx, &SIMPLE_GLOBAL_CLASS,
                                        ptr::null_mut(), h_option, &c_option);
        let global_root = Rooted::new(cx, global);
        let global = global_root.handle();

        let _ac = JSAutoCompartment::new(cx, global.get());

        let mut rval = RootedValue::new(cx, UndefinedValue());

        let orig_vec: Vec<f32> = vec![1.0, 2.9, 3.0];
        orig_vec.to_jsval(cx, rval.handle_mut());
        let converted = Vec::<f32>::from_jsval(cx, rval.handle(), ()).unwrap();

        assert_eq!(orig_vec, converted);

        let orig_vec: Vec<i32> = vec![1, 2, 3];
        orig_vec.to_jsval(cx, rval.handle_mut());
        let converted = Vec::<i32>::from_jsval(cx, rval.handle(),
                                               ConversionBehavior::Default).unwrap();

        assert_eq!(orig_vec, converted);
    }
}
