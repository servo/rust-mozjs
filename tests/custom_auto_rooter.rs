/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[macro_use]
extern crate mozjs;
use mozjs::jsapi::JSTracer;
use mozjs::jsapi::JS_GC;
use mozjs::rust::Runtime;
use mozjs::rust::CustomTrace;
use mozjs::rust::CustomAutoRooter;

macro_rules! checked_trace_impl {
    ($flag:ident, $implementer:ident) => {
        static mut $flag: bool = false;
        unsafe impl CustomTrace for $implementer {
            fn trace(&self, _: *mut JSTracer) { unsafe { $flag = true; } }
        }
    };
}

/// Check if Rust reimplementation of CustomAutoRooter properly appends itself
/// to autoGCRooters stack list and if C++ inheritance was properly simulated
/// by checking if appropriate virtual trace function was called.
#[test]
fn virtual_trace_called() {
    pub struct TestStruct { }
    checked_trace_impl!(TRACE_FN_WAS_CALLED, TestStruct);

    let rt = Runtime::new().unwrap();
    let (rt, cx) = (rt.rt(), rt.cx());

    let mut rooter = CustomAutoRooter::new(TestStruct { });
    let _guard = rooter.root(cx);

    unsafe {
        JS_GC(rt);

        assert!(TRACE_FN_WAS_CALLED);
    }
}

#[test]
fn sequence_macro() {
    pub struct TestStruct { }
    checked_trace_impl!(TRACE_FN_WAS_CALLED, TestStruct);

    let rt = Runtime::new().unwrap();
    let (rt, cx) = (rt.rt(), rt.cx());

    rooted_seq!(in(cx) let _val = vec![TestStruct { }]);

    unsafe {
        JS_GC(rt);

        assert!(TRACE_FN_WAS_CALLED);
    }
}
