/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

extern crate js;

use js::conversions::ConversionBehavior;
use js::conversions::FromJSValConvertible;
use js::jsapi::CompartmentOptions;
use js::jsapi::JS_Init;
use js::jsapi::JS_InitStandardClasses;
use js::jsapi::JS_NewGlobalObject;
use js::jsapi::JSAutoCompartment;
use js::jsapi::OnNewGlobalHookOption;
use js::jsapi::RootedObject;
use js::jsapi::RootedValue;
use js::jsval::UndefinedValue;
use js::rust::{Runtime, SIMPLE_GLOBAL_CLASS};

use std::ptr;

#[test]
fn evaluate() {
    unsafe {
        assert!(JS_Init());
        let rt = Runtime::new();
        let cx = rt.cx();

        let global = RootedObject::new(cx,
            JS_NewGlobalObject(cx, &SIMPLE_GLOBAL_CLASS, ptr::null_mut(),
                               OnNewGlobalHookOption::FireOnNewGlobalHook,
                               &CompartmentOptions::default())
        );
        let _ac = JSAutoCompartment::new(cx, global.handle().get());
        assert!(JS_InitStandardClasses(cx, global.handle()));
        let mut rval = RootedValue::new(cx, UndefinedValue());
        rt.evaluate_script(global.handle(), "new Set([1, 2, 3])",
                           "test", 1, rval.handle_mut()).unwrap();
        let converted =
          Vec::<i32>::from_jsval(cx, rval.handle(),
                                 ConversionBehavior::Default).unwrap();
        assert_eq!(&[1, 2, 3], &*converted);
    }
}
