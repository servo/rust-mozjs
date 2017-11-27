/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![cfg(feature = "debugmozjs")]

#[macro_use]
extern crate mozjs;
use mozjs::jsapi::JSTracer;
use mozjs::rust::Runtime as Runtime_;
use mozjs::rust::CustomTrace;
use mozjs::jsapi::JS_GC;
use mozjs::jsapi::RootKind;
use mozjs::rust::RootKind as RootKind_;
use mozjs::rust::GCMethods;

/// Check if Rust reimplementation of CustomAutoRooter properly appends itself
/// to autoGCRooters stack list and if C++ inheritance was properly simulated
/// by checking if appropriate virtual trace function was called.
#[test]
fn custom_auto_rooter_vftable() {
    static mut TRACE_FN_WAS_CALLED: bool = false;

    pub struct TestStruct { }

    unsafe impl CustomTrace for TestStruct {
        fn trace(&self, _: *mut JSTracer) {
            unsafe { TRACE_FN_WAS_CALLED = true; }
        }
    }

    let rt = Runtime_::new().unwrap();

    unsafe {
        let mut rooted = mozjs::rust::CustomAutoRooter::<TestStruct>::new();
        rooted.data = Some(TestStruct { });

        rooted.add_to_root_stack(rt.cx());
        JS_GC(rt.rt());
        rooted.remove_from_root_stack();

        assert!(TRACE_FN_WAS_CALLED);
    }
}

/// Similar to `custom_auto_rooter_vftable` test, this checks if appropriate
/// C++ virtual trace function is called on Rust side.
#[test]
fn sequence_rooter_vftable() {
    static mut TRACE_FN_WAS_CALLED: bool = false;

    pub struct TestStruct { }

    unsafe impl CustomTrace for TestStruct {
        fn trace(&self, _: *mut JSTracer) {
            unsafe { TRACE_FN_WAS_CALLED = true; }
        }
    }
    impl GCMethods for TestStruct {
        unsafe fn initial() -> Self { TestStruct { } }
        unsafe fn post_barrier(_: *mut Self, _: Self, _: Self) { }
    }
    impl RootKind_ for TestStruct { fn rootKind() -> RootKind { RootKind::Object } }

    let rt = Runtime_::new().unwrap();

    unsafe {
        rooted_seq!(in(rt.cx()) let _value = vec![TestStruct { }]);
        JS_GC(rt.rt());

        assert!(TRACE_FN_WAS_CALLED);
    }
}

