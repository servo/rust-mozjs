/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/. */

#![crate_name = "js"]
#![crate_type = "rlib"]

#![feature(globs, link_args, phase, unsafe_destructor)]

#![allow(non_upper_case_globals, non_camel_case_types, non_snake_case, improper_ctypes)]

#![reexport_test_harness_main = "test_main"]

extern crate green;
extern crate libc;
#[phase(plugin, link)]
extern crate log;
#[cfg(test)]
extern crate rustuv;
extern crate serialize;

use libc::c_uint;
use libc::types::common::c99::uint32_t;
use jsapi::{JSBool, JSContext, JSPropertyOp, JSStrictPropertyOp, JSEnumerateOp,
            JSObject, jsid, JSResolveOp, JSConvertOp, JSFinalizeOp, JSTraceOp,
            JSProto_LIMIT, JSHandleObject, JSCheckAccessOp, JSNative, JSHasInstanceOp};
use jsapi::JS_ComputeThis;
use jsval::JSVal;

// These are just macros in jsapi.h
pub use jsapi::JS_Init as JS_NewRuntime;
pub use jsapi::JS_Finish as JS_DestroyRuntime;
/*
FIXME: Not sure where JS_Lock is
pub use jsapi::bindgen::JS_Lock as JS_LockRuntime;
pub use jsapi::bindgen::JS_Unlock as JS_UnlockRuntime;
*/

pub use jsfriendapi::JSJitInfo;

pub mod jsapi;
pub mod linkhack;
pub mod rust;
pub mod glue;
pub mod trace;
pub mod jsval;
pub mod jsfriendapi;

// FIXME: Add the remaining options
pub const JSOPTION_STRICT: uint32_t =    0b00000000000001u32;
pub const JSOPTION_WERROR: uint32_t =    0b00000000000010u32;
pub const JSOPTION_VAROBJFIX: uint32_t = 0b00000000000100u32;
pub const JSOPTION_METHODJIT: uint32_t = (1u32 << 14) as u32;
pub const JSOPTION_TYPE_INFERENCE: uint32_t = (1u32 << 18) as u32;

pub const default_heapsize: u32 = 32_u32 * 1024_u32 * 1024_u32;
pub const default_stacksize: uint = 8192u;
pub const ERR: JSBool = 0_i32;

pub const JSID_TYPE_STRING: i64 = 0;
pub const JSID_TYPE_INT: i64 = 1;
pub const JSID_TYPE_VOID: i64 = 2;
pub const JSID_TYPE_OBJECT: i64 = 4;
pub const JSID_TYPE_DEFAULT_XML_NAMESPACE: i64 = 6;
pub const JSID_TYPE_MASK: i64 = 7;

pub const JSID_VOID: jsid = JSID_TYPE_VOID as jsid;

pub const JSFUN_CONSTRUCTOR: u32 = 0x200; /* native that can be called as a ctor */

pub const JSPROP_ENUMERATE: c_uint = 0x01;
pub const JSPROP_READONLY: c_uint  = 0x02;
pub const JSPROP_PERMANENT: c_uint = 0x04;
pub const JSPROP_GETTER: c_uint = 0x10;
pub const JSPROP_SETTER: c_uint = 0x20;
pub const JSPROP_SHARED: c_uint =    0x40;
pub const JSPROP_NATIVE_ACCESSORS: c_uint = 0x08;

pub const JSCLASS_RESERVED_SLOTS_SHIFT: c_uint = 8;
pub const JSCLASS_RESERVED_SLOTS_WIDTH: c_uint = 8;
pub const JSCLASS_RESERVED_SLOTS_MASK: c_uint = ((1u << JSCLASS_RESERVED_SLOTS_WIDTH as uint) - 1) as c_uint;

pub const JSCLASS_HIGH_FLAGS_SHIFT: c_uint =
    JSCLASS_RESERVED_SLOTS_SHIFT + JSCLASS_RESERVED_SLOTS_WIDTH;
pub const JSCLASS_IS_GLOBAL: c_uint = (1<<((JSCLASS_HIGH_FLAGS_SHIFT as uint)+1));

pub const JSCLASS_GLOBAL_SLOT_COUNT: c_uint = JSProto_LIMIT * 3 + 24;

pub const JSCLASS_IS_DOMJSCLASS: u32 = 1 << 4;
pub const JSCLASS_USERBIT1: u32 = 1 << 7;

pub const JSSLOT_PROXY_PRIVATE: u32 = 1;

pub const JSRESOLVE_QUALIFIED: u32 = 0x01;
pub const JSRESOLVE_ASSIGNING: u32 = 0x02;
pub const JSRESOLVE_DETECTING: u32 = 0x04;

pub enum JSGCTraceKind {
    JSTRACE_OBJECT,
    JSTRACE_STRING,
    JSTRACE_SCRIPT
}

pub fn JSCLASS_HAS_RESERVED_SLOTS(n: c_uint) -> c_uint {
    (n & JSCLASS_RESERVED_SLOTS_MASK) << (JSCLASS_RESERVED_SLOTS_SHIFT as uint)
}

#[inline(always)]
pub unsafe fn JS_ARGV(_cx: *mut JSContext, vp: *mut JSVal) -> *mut JSVal {
    vp.offset(2)
}

pub unsafe fn JS_SET_RVAL(_cx: *mut JSContext, vp: *mut JSVal, v: JSVal) {
    *vp = v;
}

#[inline(alwyas)]
pub unsafe fn JS_THIS_OBJECT(cx: *mut JSContext, vp: *mut JSVal) -> *mut JSObject {
    let r =
        if (*(vp.offset(1))).is_primitive() {
            JS_ComputeThis(cx, vp)
        } else {
            *(vp.offset(1))
        };
    r.to_object_or_null()
}

#[inline(always)]
pub unsafe fn JS_CALLEE(_cx: *mut JSContext, vp: *mut JSVal) -> JSVal {
    *vp
}

// Run tests with libgreen instead of libnative.
#[cfg(test)]
#[start]
fn start(argc: int, argv: *const *const u8) -> int {
    green::start(argc, argv, rustuv::event_loop, test_main)
}

pub type JSObjectOp = extern "C" fn(*mut JSContext, JSHandleObject) -> *mut JSObject;

pub struct Class {
    pub name: *const libc::c_char,
    pub flags: uint32_t,
    pub addProperty: JSPropertyOp,
    pub delProperty: JSPropertyOp,
    pub getProperty: JSPropertyOp,
    pub setProperty: JSStrictPropertyOp,
    pub enumerate: JSEnumerateOp,
    pub resolve: JSResolveOp,
    pub convert: JSConvertOp,
    pub finalize: JSFinalizeOp,
    pub checkAccess: JSCheckAccessOp,
    pub call: JSNative,
    pub hasInstance: JSHasInstanceOp,
    pub construct: JSNative,
    pub trace: JSTraceOp,

    pub ext: ClassExtension,
    pub ops: ObjectOps,
}

pub struct ClassExtension {
    pub equality: *const u8,
    pub outerObject: Option<JSObjectOp>,
    pub innerObject: Option<JSObjectOp>,
    pub iteratorObject: *const u8,
    pub unused: *const u8,
    pub isWrappedNative: *const u8,
}

pub struct ObjectOps {
    pub lookupGeneric: *const u8,
    pub lookupProperty: *const u8,
    pub lookupElement: *const u8,
    pub lookupSpecial: *const u8,
    pub defineGeneric: *const u8,
    pub defineProperty: *const u8,
    pub defineElement: *const u8,
    pub defineSpecial: *const u8,
    pub getGeneric: *const u8,
    pub getProperty: *const u8,
    pub getElement: *const u8,
    pub getElementIfPresent: *const u8,
    pub getSpecial: *const u8,
    pub setGeneric: *const u8,
    pub setProperty: *const u8,
    pub setElement: *const u8,
    pub setSpecial: *const u8,
    pub getGenericAttributes: *const u8,
    pub getPropertyAttributes: *const u8,
    pub getElementAttributes: *const u8,
    pub getSpecialAttributes: *const u8,
    pub setGenericAttributes: *const u8,
    pub setPropertyAttributes: *const u8,
    pub setElementAttributes: *const u8,
    pub setSpecialAttributes: *const u8,
    pub deleteProperty: *const u8,
    pub deleteElement: *const u8,
    pub deleteSpecial: *const u8,

    pub enumerate: *const u8,
    pub typeOf: *const u8,
    pub thisObject: Option<JSObjectOp>,
    pub clear: *const u8,
}
