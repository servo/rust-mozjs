/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[macro_use]
extern crate js;

use js::glue::CallObjectTracer;
use js::jsapi::CompartmentOptions;
use js::jsapi::GCTraceKindToAscii;
use js::jsapi::Heap;
use js::jsapi::JS_AddExtraGCRootsTracer;
use js::jsapi::JSAutoCompartment;
use js::jsapi::JS_NewGlobalObject;
use js::jsapi::JSObject;
use js::jsapi::JSTracer;
use js::jsapi::OnNewGlobalHookOption;
use js::jsapi::TraceKind;
use js::jsapi::Type;
use js::jsval::UndefinedValue;
use js::rust::Runtime as Runtime_;
use js::rust::SIMPLE_GLOBAL_CLASS;
use js::typedarray::{ArrayBufferView, CreateWith, TypedArray, TypedArrayElement, Uint8Array, Uint16Array, Uint32Array};
use std::cell::RefCell;
use std::ops::{Deref, DerefMut};
use std::os::raw::c_void;
use std::ptr;

thread_local!(
    static ROOTED_TRACEABLES: RefCell<Vec<*const Heap<*mut JSObject>>> = RefCell::new(Vec::new());
);

unsafe extern fn trace(tracer: *mut JSTracer, _: *mut c_void) {
    ROOTED_TRACEABLES.with(|v| {
        for &array in &*v.borrow() {
            CallObjectTracer(tracer,
                             array as *mut _,
                             GCTraceKindToAscii(TraceKind::Object));
        }
    });
}

struct Tracer<T: TypedArrayElement> {
    array: TypedArray<T>,
}

impl<T: TypedArrayElement> Tracer<T> {
    fn new(array: TypedArray<T>) -> Self {
        let ptr = array.object() as *const _;
        ROOTED_TRACEABLES.with(|v| {
            v.borrow_mut().push(ptr);
        });
        Tracer {
            array: array,
        }
    }
}


impl<T: TypedArrayElement> Deref for Tracer<T> {
    type Target = TypedArray<T>;
    fn deref(&self) -> &Self::Target {
        &self.array
    }
}

impl<T: TypedArrayElement> DerefMut for Tracer<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.array
    }
}

impl<T: TypedArrayElement> Drop for Tracer<T> {
    fn drop(&mut self) {
        let ptr = ROOTED_TRACEABLES.with(|v| {
            v.borrow_mut().pop()
        });
        assert_eq!(self.array.object() as *const _, ptr.unwrap());
    }
}


#[test]
fn typedarray() {
    let rt = Runtime_::new();
    let cx = rt.cx();

    unsafe {
        JS_AddExtraGCRootsTracer(rt.rt(), Some(trace), ptr::null_mut());

        rooted!(in(cx) let global =
            JS_NewGlobalObject(cx, &SIMPLE_GLOBAL_CLASS, ptr::null_mut(),
                               OnNewGlobalHookOption::FireOnNewGlobalHook,
                               &CompartmentOptions::default())
        );

        let _ac = JSAutoCompartment::new(cx, global.get());

        rooted!(in(cx) let mut rval = UndefinedValue());
        assert!(rt.evaluate_script(global.handle(), "new Uint8Array([0, 2, 4])",
                                   "test", 1, rval.handle_mut()).is_ok());
        assert!(rval.is_object());

        let mut array = Tracer::new(Uint8Array::from(cx, rval.to_object()).unwrap());
        assert_eq!(array.as_slice(), &[0, 2, 4][..]);

        let array = Uint16Array::from(cx, rval.to_object());
        assert!(array.is_err());

        let view = Tracer::new(ArrayBufferView::from(cx, rval.to_object()).unwrap());
        assert_eq!(view.get_array_type(), Type::Uint8);

        rooted!(in(cx) let mut rval = ptr::null_mut());
        assert!(Uint32Array::create(cx, CreateWith::Slice(&[1, 3, 5]), rval.handle_mut()).is_ok());

        let mut array = Tracer::new(Uint32Array::from(cx, rval.get()).unwrap());
        assert_eq!(array.as_slice(), &[1, 3, 5][..]);

        let mut array = Tracer::new(Uint32Array::from(cx, rval.get()).unwrap());
        array.update(&[2, 4, 6]);
        assert_eq!(array.as_slice(), &[2, 4, 6][..]);

        rooted!(in(cx) let rval = ptr::null_mut());
        let array = Uint8Array::from(cx, rval.get());
        assert!(array.is_err());

        rooted!(in(cx) let mut rval = ptr::null_mut());
        assert!(Uint32Array::create(cx, CreateWith::Length(5), rval.handle_mut()).is_ok());

        let mut array = Tracer::new(Uint32Array::from(cx, rval.get()).unwrap());
        assert_eq!(array.as_slice(), &[0, 0, 0, 0, 0]);

        let mut array = Tracer::new(Uint32Array::from(cx, rval.get()).unwrap());
        array.update(&[0, 1, 2, 3]);
        assert_eq!(array.as_slice(), &[0, 1, 2, 3, 0]);

        let view = Tracer::new(ArrayBufferView::from(cx, rval.get()).unwrap());
        assert_eq!(view.get_array_type(), Type::Uint32);
    }
}

#[test]
#[should_panic]
fn typedarray_update_panic() {
    let rt = Runtime_::new();
    let cx = rt.cx();

    unsafe {
        JS_AddExtraGCRootsTracer(rt.rt(), Some(trace), ptr::null_mut());

        rooted!(in(cx) let global =
            JS_NewGlobalObject(cx, &SIMPLE_GLOBAL_CLASS, ptr::null_mut(),
                               OnNewGlobalHookOption::FireOnNewGlobalHook,
                               &CompartmentOptions::default())
        );

        let _ac = JSAutoCompartment::new(cx, global.get());
        rooted!(in(cx) let mut rval = ptr::null_mut());
        let _ = Uint32Array::create(cx, CreateWith::Slice(&[1, 2, 3, 4, 5]), rval.handle_mut());
        let mut array = Tracer::new(Uint32Array::from(cx, rval.get()).unwrap());
        array.update(&[0, 2, 4, 6, 8, 10]);
    }
}
