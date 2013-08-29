/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/. */

#[doc = "

Handy functions for creating class objects and so forth.

"];

use glue::GetJSClassHookStubPointer;
use glue::{PROPERTY_STUB, STRICT_PROPERTY_STUB, ENUMERATE_STUB,
              RESOLVE_STUB, CONVERT_STUB};
use std::libc::c_uint;
use std::str::raw::from_c_str;
use std::cast::transmute;
use name_pool::*;
use std::ptr::null;
use std::ptr;
use jsapi;
use jsapi::{JSClass, JSContext, JSVal, JSFunctionSpec, JSBool, JSNativeWrapper};
use jsapi::{JS_EncodeString, JS_free, JS_ValueToBoolean, JS_ValueToString};
use jsapi::{JS_ReportError, JS_ValueToSource};
use JSCLASS_IS_GLOBAL;
use JSCLASS_HAS_RESERVED_SLOTS;
use JSCLASS_GLOBAL_SLOT_COUNT;
use JS_ARGV;
use JSVAL_VOID;
use JS_SET_RVAL;

pub fn basic_class(np: @mut NamePool, name: ~str) -> JSClass {
    JSClass {
        name: np.add(name),
        flags: JSCLASS_IS_GLOBAL | JSCLASS_HAS_RESERVED_SLOTS(JSCLASS_GLOBAL_SLOT_COUNT + 1),
        addProperty: unsafe { GetJSClassHookStubPointer(PROPERTY_STUB) as *u8 },
        delProperty: unsafe { GetJSClassHookStubPointer(PROPERTY_STUB) as *u8 },
        getProperty: unsafe { GetJSClassHookStubPointer(PROPERTY_STUB) as *u8 },
        setProperty: unsafe { GetJSClassHookStubPointer(STRICT_PROPERTY_STUB) as *u8 },
        enumerate: unsafe { GetJSClassHookStubPointer(ENUMERATE_STUB) as *u8 },
        resolve: unsafe { GetJSClassHookStubPointer(RESOLVE_STUB) as *u8 },
        convert: unsafe { GetJSClassHookStubPointer(CONVERT_STUB) as *u8 },
        finalize: null(),
        checkAccess: null(),
        call: null(),
        hasInstance: null(),
        construct: null(),
        trace: null(),
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

pub fn global_class(np: @mut NamePool) -> JSClass {
    basic_class(np, ~"global")
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

pub extern fn debug(cx: *JSContext, argc: c_uint, vp: *JSVal) -> JSBool {
    unsafe {
        let argv = JS_ARGV(cx, vp);
        for i in range(0, argc as int) {
            let jsstr = JS_ValueToString(cx, *ptr::offset(argv, i));
            debug!("%s", jsval_to_rust_str(cx, jsstr));
        }
        JS_SET_RVAL(cx, vp, JSVAL_VOID);
        return 1_i32;
    }
}

pub extern fn assert(cx: *JSContext, argc: c_uint, vp: *JSVal) -> JSBool {
    unsafe {
        let argv = JS_ARGV(cx, vp);

        let argument = match argc {
            0 => JSVAL_VOID,
            _ => *ptr::offset(argv, 0)
        };

        let result = 0;
        if JS_ValueToBoolean(cx, argument, &result) == 0 {
            return 0_i32;
        }

        if result == 0 {
            // This operation can fail, but that is not critical.
            let source = JS_ValueToSource(cx, argument);
            let msg = fmt!("JavaScript assertion failed: %s is falsy!",
                            jsval_to_rust_str(cx, source));

            debug!(msg);
            do msg.to_c_str().with_ref |buf| {
              JS_ReportError(cx, buf);
            }
            return 0_i32;
        }

        JS_SET_RVAL(cx, vp, JSVAL_VOID);
        return 1_i32;
    }
}

pub fn debug_fns(np: @mut NamePool) -> ~[JSFunctionSpec] {
    ~[
        JSFunctionSpec {
            name: np.add(~"debug"),
            call: JSNativeWrapper {
                op: debug,
                info: null()
            },
            nargs: 0,
            flags: 0,
            selfHostedName: null()
        },
        JSFunctionSpec {
            name: np.add(~"assert"),
            call: JSNativeWrapper {
                op: assert,
                info: null()
            },
            nargs: 1,
            flags: 0,
            selfHostedName: null()
        },
        JSFunctionSpec {
            name: null(),
            call: JSNativeWrapper {
                op: null(),
                info: null(),
            },
            nargs: 0,
            flags: 0,
            selfHostedName: null()
        }
    ]
}

