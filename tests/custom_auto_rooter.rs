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
use std::cell::Cell;

struct TraceCheck {
    trace_was_called: Cell<bool>
}

impl TraceCheck {
    fn new() -> TraceCheck {
        TraceCheck { trace_was_called: Cell::new(false) }
    }
}

unsafe impl CustomTrace for TraceCheck {
    fn trace(&self, _: *mut JSTracer) {
        self.trace_was_called.set(true);
    }
}

/// Check if Rust reimplementation of CustomAutoRooter properly appends itself
/// to autoGCRooters stack list and if C++ inheritance was properly simulated
/// by checking if appropriate virtual trace function was called.
#[test]
fn virtual_trace_called() {
    let rt = Runtime::new().unwrap();
    let (rt, cx) = (rt.rt(), rt.cx());

    let mut rooter = CustomAutoRooter::new(TraceCheck::new());
    let guard = rooter.root(cx);

    unsafe { JS_GC(rt); }

    assert!(guard.trace_was_called.get());
}

#[test]
fn root_macro() {
    let rt = Runtime::new().unwrap();
    let (rt, cx) = (rt.rt(), rt.cx());

    auto_root!(in(cx) let vec = vec![TraceCheck::new(), TraceCheck::new()]);

    unsafe { JS_GC(rt); }

    vec.iter().for_each(|elem| assert!(elem.trace_was_called.get()));
}
