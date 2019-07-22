/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[macro_use]
extern crate mozjs;

use mozjs::jsapi::JSAutoRealm;
use mozjs::jsapi::JSObject;
use mozjs::jsapi::JS_NewGlobalObject;
use mozjs::jsapi::OnNewGlobalHookOption;
use mozjs::jsapi::Type;
use mozjs::jsval::UndefinedValue;
use mozjs::rust::JSEngine;
use mozjs::rust::RealmOptions;
use mozjs::rust::Runtime;
use mozjs::rust::SIMPLE_GLOBAL_CLASS;
use mozjs::typedarray::{CreateWith, Uint32Array};
use std::ptr;

#[test]
fn typedarray() {
    let engine = JSEngine::init().unwrap();
    let rt = Runtime::new(engine);
    let cx = rt.cx();

    unsafe {
        let options = RealmOptions::default();
        rooted!(in(cx) let global =
            JS_NewGlobalObject(cx, &SIMPLE_GLOBAL_CLASS, ptr::null_mut(),
                               OnNewGlobalHookOption::FireOnNewGlobalHook,
                               &*options)
        );

        let _ac = JSAutoRealm::new(cx, global.get());

        rooted!(in(cx) let mut rval = UndefinedValue());
        assert!(rt.evaluate_script(global.handle(), "new Uint8Array([0, 2, 4])",
                                   "test", 1, rval.handle_mut()).is_ok());
        assert!(rval.is_object());

        typedarray!(in(cx) let array: Uint8Array = rval.to_object());
        assert_eq!(array.unwrap().as_slice(), &[0, 2, 4][..]);

        typedarray!(in(cx) let array: Uint8Array = rval.to_object());
        assert_eq!(array.unwrap().len(), 3);

        typedarray!(in(cx) let array: Uint8Array = rval.to_object());
        assert_eq!(array.unwrap().to_vec(), vec![0, 2, 4]);

        typedarray!(in(cx) let array: Uint16Array = rval.to_object());
        assert!(array.is_err());

        typedarray!(in(cx) let view: ArrayBufferView = rval.to_object());
        assert_eq!(view.unwrap().get_array_type(), Type::Uint8);

        rooted!(in(cx) let mut rval = ptr::null_mut::<JSObject>());
        assert!(Uint32Array::create(cx, CreateWith::Slice(&[1, 3, 5]), rval.handle_mut()).is_ok());

        typedarray!(in(cx) let array: Uint32Array = rval.get());
        assert_eq!(array.unwrap().as_slice(), &[1, 3, 5][..]);

        typedarray!(in(cx) let mut array: Uint32Array = rval.get());
        array.as_mut().unwrap().update(&[2, 4, 6]);
        assert_eq!(array.unwrap().as_slice(), &[2, 4, 6][..]);

        rooted!(in(cx) let rval = ptr::null_mut::<JSObject>());
        typedarray!(in(cx) let array: Uint8Array = rval.get());
        assert!(array.is_err());

        rooted!(in(cx) let mut rval = ptr::null_mut::<JSObject>());
        assert!(Uint32Array::create(cx, CreateWith::Length(5), rval.handle_mut()).is_ok());

        typedarray!(in(cx) let array: Uint32Array = rval.get());
        assert_eq!(array.unwrap().as_slice(), &[0, 0, 0, 0, 0]);

        typedarray!(in(cx) let mut array: Uint32Array = rval.get());
        array.as_mut().unwrap().update(&[0, 1, 2, 3]);
        assert_eq!(array.unwrap().as_slice(), &[0, 1, 2, 3, 0]);

        typedarray!(in(cx) let view: ArrayBufferView = rval.get());
        assert_eq!(view.unwrap().get_array_type(), Type::Uint32);

        typedarray!(in(cx) let view: ArrayBufferView = rval.get());
        assert_eq!(view.unwrap().is_shared(), false);
    }
}
