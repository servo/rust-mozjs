/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[macro_use]
extern crate mozjs;
use mozjs::jsapi::GCReason;
use mozjs::jsapi::JSTracer;
use mozjs::jsapi::JS_GC;
use mozjs::rust::JSEngine;
use mozjs::rust::Runtime;
use mozjs::rust::CustomTrace;
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

#[test]
fn custom_auto_rooter_macro() {
    let engine = JSEngine::init().unwrap();
    let rt = Runtime::new(engine);
    let cx = rt.cx();

    auto_root!(in(cx) let vec = vec![TraceCheck::new(), TraceCheck::new()]);

    unsafe { JS_GC(cx, GCReason::API); }

    vec.iter().for_each(|elem| assert!(elem.trace_was_called.get()));
}
