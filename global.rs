/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/. */

#[doc = "

Handy functions for creating class objects and so forth.

"];

use glue::bindgen::GetJSClassHookStubPointer;
use glue::{PROPERTY_STUB, STRICT_PROPERTY_STUB, ENUMERATE_STUB,
              RESOLVE_STUB, CONVERT_STUB};
use core::libc::c_uint;
use core::str::raw::from_c_str;
use core::cast::reinterpret_cast;
use name_pool::*;
use core::ptr::null;
use jsapi;
use jsapi::{JSClass, JSContext, JSVal, JSFunctionSpec, JSBool, JSNativeWrapper};
use jsapi::bindgen::{JS_EncodeString, JS_free, JS_ValueToString};
use JSCLASS_IS_GLOBAL;
use JSCLASS_HAS_RESERVED_SLOTS;
use JSCLASS_GLOBAL_SLOT_COUNT;
use JS_ARGV;
use JSVAL_NULL;
use JS_SET_RVAL;

pub fn basic_class(np: @mut NamePool, name: ~str) -> JSClass {
    JSClass {
        name: np.add(name),
        flags: JSCLASS_IS_GLOBAL | JSCLASS_HAS_RESERVED_SLOTS(JSCLASS_GLOBAL_SLOT_COUNT),
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
        JS_free(cx, reinterpret_cast(&bytes));
        s
    }
}

pub extern fn debug(cx: *JSContext, argc: c_uint, vp: *JSVal) -> JSBool {
    unsafe {
        let argv = JS_ARGV(cx, vp);
        for uint::range(0u, argc as uint) |i| {
            let jsstr = JS_ValueToString(cx, *ptr::offset(argv, i));
            debug!("%s", jsval_to_rust_str(cx, jsstr));
        }
        JS_SET_RVAL(cx, vp, JSVAL_NULL);
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

