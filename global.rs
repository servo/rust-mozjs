/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/. */

#![doc = "

Handy functions for creating class objects and so forth.

"]

use std::libc::{c_uint, c_void};
use std::str::raw::from_c_str;
use std::cast::transmute;
use std::ptr::null;
use jsapi;
use jsapi::{JSClass, JSContext, JSFunctionSpec, JSBool, JSNativeWrapper, JS_EnumerateStub};
use jsapi::{JS_EncodeString, JS_free, JS_ValueToBoolean, JS_ValueToString, JS_ConvertStub};
use jsapi::{JS_ReportError, JS_ValueToSource, JS_GC, JS_GetRuntime, JS_PropertyStub};
use jsapi::{JS_StrictPropertyStub, JS_ResolveStub};
use jsfriendapi::JSJitInfo;
use jsval::{JSVal, UndefinedValue};
use JSCLASS_IS_GLOBAL;
use JSCLASS_HAS_RESERVED_SLOTS;
use JSCLASS_RESERVED_SLOTS_MASK;
use JSCLASS_RESERVED_SLOTS_SHIFT;
use JSCLASS_GLOBAL_SLOT_COUNT;
use JS_ARGV;
use JS_SET_RVAL;

static global_name: [i8, ..7] = ['g' as i8, 'l' as i8, 'o' as i8, 'b' as i8, 'a' as i8, 'l' as i8, 0 as i8];
pub static BASIC_GLOBAL: JSClass = JSClass {
    name: &global_name as *i8,
        flags: JSCLASS_IS_GLOBAL | (((JSCLASS_GLOBAL_SLOT_COUNT + 1) & JSCLASS_RESERVED_SLOTS_MASK) << JSCLASS_RESERVED_SLOTS_SHIFT),
        addProperty: Some(JS_PropertyStub),
        delProperty: Some(JS_PropertyStub),
        getProperty: Some(JS_PropertyStub),
        setProperty: Some(JS_StrictPropertyStub),
        enumerate: Some(JS_EnumerateStub),
        resolve: Some(JS_ResolveStub),
        convert: Some(JS_ConvertStub),
        finalize: None,
        checkAccess: None,
        call: None,
        hasInstance: None,
        construct: None,
        trace: None,
        reserved: (0 as *c_void, 0 as *c_void, 0 as *c_void, 0 as *c_void, 0 as *c_void,  // 05
                   0 as *c_void, 0 as *c_void, 0 as *c_void, 0 as *c_void, 0 as *c_void,  // 10
                   0 as *c_void, 0 as *c_void, 0 as *c_void, 0 as *c_void, 0 as *c_void,  // 15
                   0 as *c_void, 0 as *c_void, 0 as *c_void, 0 as *c_void, 0 as *c_void,  // 20
                   0 as *c_void, 0 as *c_void, 0 as *c_void, 0 as *c_void, 0 as *c_void,  // 25
                   0 as *c_void, 0 as *c_void, 0 as *c_void, 0 as *c_void, 0 as *c_void,  // 30
                   0 as *c_void, 0 as *c_void, 0 as *c_void, 0 as *c_void, 0 as *c_void,  // 35
                   0 as *c_void, 0 as *c_void, 0 as *c_void, 0 as *c_void, 0 as *c_void)  // 40
};

pub fn basic_class(name: &'static str) -> JSClass {
    JSClass {
        name: name.as_ptr() as *i8,
        flags: JSCLASS_IS_GLOBAL | JSCLASS_HAS_RESERVED_SLOTS(JSCLASS_GLOBAL_SLOT_COUNT + 1),
        addProperty: Some(JS_PropertyStub),
        delProperty: Some(JS_PropertyStub),
        getProperty: Some(JS_PropertyStub),
        setProperty: Some(JS_StrictPropertyStub),
        enumerate: Some(JS_EnumerateStub),
        resolve: Some(JS_ResolveStub),
        convert: Some(JS_ConvertStub),
        finalize: None,
        checkAccess: None,
        call: None,
        hasInstance: None,
        construct: None,
        trace: None,
        reserved: (null(), null(), null(), null(), null(),  // 05
                   null(), null(), null(), null(), null(),  // 10
                   null(), null(), null(), null(), null(),  // 15
                   null(), null(), null(), null(), null(),  // 20
                   null(), null(), null(), null(), null(),  // 25
                   null(), null(), null(), null(), null(),  // 30
                   null(), null(), null(), null(), null(),  // 35
                   null(), null(), null(), null(), null())  // 40
    }
}

pub fn global_class() -> JSClass {
    basic_class("global")
}

pub unsafe fn jsval_to_rust_str(cx: *JSContext, vp: *jsapi::JSString) -> ~str {
    if vp.is_null() {
        ~""
    } else {
        let bytes = JS_EncodeString(cx, vp);
        let s = from_c_str(bytes);
        JS_free(cx, transmute(bytes));
        s
    }
}

pub extern fn debug(cx: *JSContext, argc: c_uint, vp: *mut JSVal) -> JSBool {
    unsafe {
        let argv = JS_ARGV(cx, &*vp);
        for i in range(0, argc as int) {
            let jsstr = JS_ValueToString(cx, *argv.offset(i));
            debug!("{:s}", jsval_to_rust_str(cx, jsstr));
        }
        JS_SET_RVAL(cx, &*vp, UndefinedValue());
        return 1_i32;
    }
}

pub extern fn gc(cx: *JSContext, _argc: c_uint, vp: *mut JSVal) -> JSBool {
    unsafe {
        JS_GC(JS_GetRuntime(cx));
        JS_SET_RVAL(cx, &*vp, UndefinedValue());
        return 1;
    }
}


pub extern fn assert(cx: *JSContext, argc: c_uint, vp: *mut JSVal) -> JSBool {
    unsafe {
        let argv = JS_ARGV(cx, &*vp);

        let argument = match argc {
            0 => UndefinedValue(),
            _ => *argv.offset(0)
        };

        let result = 0;
        if JS_ValueToBoolean(cx, argument, &result) == 0 {
            return 0_i32;
        }

        if result == 0 {
            // This operation can fail, but that is not critical.
            let source = JS_ValueToSource(cx, argument);
            let msg = format!("JavaScript assertion failed: {:s} is falsy!",
                              jsval_to_rust_str(cx, source));

            debug!("{:s}", msg);
            msg.to_c_str().with_ref(|buf| {
              JS_ReportError(cx, buf);
            });
            return 0_i32;
        }

        JS_SET_RVAL(cx, &*vp, UndefinedValue());
        return 1_i32;
    }
}

static debug_name: [i8, ..6] = ['d' as i8, 'e' as i8, 'b' as i8, 'u' as i8, 'g' as i8, 0 as i8];
static assert_name: [i8, ..7] = ['a' as i8, 's' as i8, 's' as i8, 'e' as i8, 'r' as i8, 't' as i8, 0 as i8];
static gc_name: [i8, ..3] = ['g' as i8, 'c' as i8, 0 as i8];

pub static DEBUG_FNS: &'static [JSFunctionSpec] = &[
    JSFunctionSpec {
        name: &debug_name as *i8,
        call: JSNativeWrapper {
            op: Some(debug),
            info: 0 as *JSJitInfo
        },
        nargs: 0,
        flags: 0,
        selfHostedName: 0 as *i8
    },
    JSFunctionSpec {
        name: &assert_name as *i8,
        call: JSNativeWrapper {
            op: Some(assert),
            info: 0 as *JSJitInfo
        },
        nargs: 0,
        flags: 0,
        selfHostedName: 0 as *i8
    },
    JSFunctionSpec {
        name: &gc_name as *i8,
        call: JSNativeWrapper {
            op: Some(gc),
            info: 0 as *JSJitInfo
        },
        nargs: 0,
        flags: 0,
        selfHostedName: 0 as *i8
    },
    JSFunctionSpec {
        name: 0 as *i8,
        call: JSNativeWrapper {
            op: None,
            info: 0 as *JSJitInfo,
        },
        nargs: 0,
        flags: 0,
        selfHostedName: 0 as *i8
    }
];
