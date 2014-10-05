/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/. */

#![crate_name = "js"]
#![crate_type = "rlib"]

#![feature(globs, link_args, managed_boxes, phase, unsafe_destructor)]

#![allow(non_uppercase_statics, non_camel_case_types, non_snake_case)]

#![reexport_test_harness_main = "test_main"]

extern crate green;
extern crate libc;
#[phase(plugin, link)]
extern crate log;
#[cfg(test)]
extern crate rustuv;
extern crate serialize;

use libc::{c_uint, c_void};
use libc::types::common::c99::uint32_t;
use jsapi::{JSContext, JSPropertyOp, JSStrictPropertyOp, JSEnumerateOp, Enum_JSProtoKey,
            JSObject, JSResolveOp, JSConvertOp, JSFinalizeOp, JSTraceOp, JSProto_LIMIT,
            JSHandleObject, JSNative, JSHasInstanceOp, JSFunctionSpec, JSDeletePropertyOp};
use jsapi::{JSWeakmapKeyDelegateOp, JSHandleId, JSMutableHandleObject, JSHandleValue};
use jsapi::{JS_ComputeThis, JSMutableHandleValue, JSRuntime};
use jsapi::{MutableHandle, Handle};
use jsval::JSVal;

// These are just macros in jsapi.h
pub use jsapi::JS_Init as JS_NewRuntime;
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
pub mod jsval;
pub mod jsfriendapi;

// FIXME: Add the remaining options
pub static JSOPTION_STRICT: uint32_t =    0b00000000000001u32;
pub static JSOPTION_WERROR: uint32_t =    0b00000000000010u32;
pub static JSOPTION_VAROBJFIX: uint32_t = 0b00000000000100u32;
pub static JSOPTION_METHODJIT: uint32_t = (1u32 << 14) as u32;
pub static JSOPTION_TYPE_INFERENCE: uint32_t = (1u32 << 18) as u32;

pub static default_heapsize: u32 = 32_u32 * 1024_u32 * 1024_u32;
pub static default_stacksize: uint = 8192u;

pub static JSID_TYPE_STRING: i64 = 0;
pub static JSID_TYPE_INT: i64 = 1;
pub static JSID_TYPE_VOID: i64 = 2;
pub static JSID_TYPE_OBJECT: i64 = 4;
pub static JSID_TYPE_DEFAULT_XML_NAMESPACE: i64 = 6;
pub static JSID_TYPE_MASK: i64 = 7;

//pub static JSID_VOID: jsid = JSID_TYPE_VOID as jsid;

pub static JSFUN_CONSTRUCTOR: u32 = 0x400; /* native that can be called as a ctor */

pub static JSPROP_ENUMERATE: c_uint = 0x01;
pub static JSPROP_READONLY: c_uint  = 0x02;
pub static JSPROP_PERMANENT: c_uint = 0x04;
pub static JSPROP_GETTER: c_uint = 0x10;
pub static JSPROP_SETTER: c_uint = 0x20;
pub static JSPROP_SHARED: c_uint =    0x40;
pub static JSPROP_NATIVE_ACCESSORS: c_uint = 0x08;

pub static NON_NATIVE: c_uint = 1<<(JSCLASS_HIGH_FLAGS_SHIFT+2);
pub static JSCLASS_IS_PROXY: c_uint =  1<<(JSCLASS_HIGH_FLAGS_SHIFT+4);
pub static PROXY_MINIMUM_SLOTS: c_uint = 4;

pub static JSCLASS_IMPLEMENTS_BARRIERS: c_uint = 1 << 5;

pub static JSCLASS_RESERVED_SLOTS_SHIFT: uint = 8;
pub static JSCLASS_RESERVED_SLOTS_WIDTH: uint = 8;
pub static JSCLASS_RESERVED_SLOTS_MASK: uint = ((1 << JSCLASS_RESERVED_SLOTS_WIDTH) - 1);

pub static JSCLASS_HIGH_FLAGS_SHIFT: uint =
    JSCLASS_RESERVED_SLOTS_SHIFT + JSCLASS_RESERVED_SLOTS_WIDTH;
pub static JSCLASS_IS_GLOBAL: c_uint = 1<<(JSCLASS_HIGH_FLAGS_SHIFT+1);

pub static JSCLASS_GLOBAL_SLOT_COUNT: c_uint = 3 + JSProto_LIMIT * 3 + 31;

pub static JSCLASS_IS_DOMJSCLASS: u32 = 1 << 4;
pub static JSCLASS_USERBIT1: u32 = 1 << 7;

pub static JSSLOT_PROXY_PRIVATE: u32 = /*1*/0; //XXXjdm wrong fo sho

pub static PROXY_PRIVATE_SLOT: u32 = 0;
pub static PROXY_HANDLER_SLOT: u32 = 1;
pub static PROXY_EXTRA_SLOT: u32 = 2;

pub static JSRESOLVE_QUALIFIED: u32 = 0x01;
pub static JSRESOLVE_ASSIGNING: u32 = 0x02;
pub static JSRESOLVE_DETECTING: u32 = 0x04;

pub static JS_DEFAULT_ZEAL_FREQ: u32 = 100;

pub enum JSGCTraceKind {
    JSTRACE_OBJECT,
    JSTRACE_STRING,
    JSTRACE_SCRIPT
}

pub fn JSCLASS_HAS_RESERVED_SLOTS(n: c_uint) -> c_uint {
    (n & (JSCLASS_RESERVED_SLOTS_MASK as c_uint)) << JSCLASS_RESERVED_SLOTS_SHIFT
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

// Up-to-date mozjs 075904f5f7ee1176f28630d1dff47820020e5928
pub type JSObjectOp = Option<unsafe extern "C" fn(*mut JSContext, JSHandleObject) -> *mut JSObject>;
pub type ClassObjectCreationOp = Option<extern "C" fn(*mut JSContext, Enum_JSProtoKey) -> *mut JSObject>;
pub type FinishClassInitOp = Option<extern "C" fn(*mut JSContext, JSHandleObject, JSHandleObject) -> bool>;

// Up-to-date mozjs 075904f5f7ee1176f28630d1dff47820020e5928
#[repr(C)]
pub struct Class {
    pub name: *const libc::c_char,
    pub flags: uint32_t,

    /* Mandatory function pointer members. */
    pub addProperty: JSPropertyOp,
    pub delProperty: JSDeletePropertyOp,
    pub getProperty: JSPropertyOp,
    pub setProperty: JSStrictPropertyOp,
    pub enumerate: JSEnumerateOp,
    pub resolve: JSResolveOp,
    pub convert: JSConvertOp,

    /* Optional members (may be null). */
    pub finalize: JSFinalizeOp,
    pub call: JSNative,
    pub hasInstance: JSHasInstanceOp,
    pub construct: JSNative,
    pub trace: JSTraceOp,

    pub spec: ClassSpec,
    pub ext: ClassExtension,
    pub ops: ObjectOps,
}

// Up-to-date mozjs 075904f5f7ee1176f28630d1dff47820020e5928
#[repr(C)]
pub struct ClassSpec {
    pub createConstructor: ClassObjectCreationOp,
    pub createPrototype: ClassObjectCreationOp,
    pub constructorFunctions: *const JSFunctionSpec,
    pub prototypeFunctions: *const JSFunctionSpec,
    pub finishInit: FinishClassInitOp,
}

// Up-to-date mozjs 075904f5f7ee1176f28630d1dff47820020e5928
#[repr(C)]
pub struct ClassExtension {
    pub outerObject: JSObjectOp,
    pub innerObject: JSObjectOp,
    pub iteratorObject: *const u8,
    pub isWrappedNative: bool,
    pub weakmapKeyDelegateOp: JSWeakmapKeyDelegateOp,
}

pub type LookupGenericOp = Option<unsafe extern "C" fn(*mut JSContext, JSHandleObject, JSHandleId,
                                                       JSMutableHandleObject, MutableHandle<*mut c_void>) -> bool>;
pub type LookupPropOp = Option<unsafe extern "C" fn(*mut JSContext, JSHandleObject, Handle<*mut c_void>,
                                                    JSMutableHandleObject, MutableHandle<*mut c_void>) -> bool>;
pub type LookupElementOp = Option<unsafe extern "C" fn(*mut JSContext, JSHandleObject, u32,
                                                       JSMutableHandleObject, MutableHandle<*mut c_void>) -> bool>;
pub type DefineGenericOp = Option<unsafe extern "C" fn(*mut JSContext, JSHandleObject, JSHandleId,
                                                       JSHandleValue, JSPropertyOp, JSStrictPropertyOp,
                                                       c_uint) -> bool>;
pub type DefinePropOp = Option<unsafe extern "C" fn(*mut JSContext, JSHandleObject, Handle<*mut c_void>,
                                                    JSHandleValue, JSPropertyOp, JSStrictPropertyOp,
                                                    c_uint) -> bool>;
pub type DefineElementOp = Option<unsafe extern "C" fn(*mut JSContext, JSHandleObject, u32, JSHandleValue,
                                                       JSPropertyOp, JSStrictPropertyOp, c_uint) -> bool>;
pub type GenericIdOp = Option<unsafe extern "C" fn(*mut JSContext, JSHandleObject, JSHandleObject,
                                                   JSHandleId, JSMutableHandleValue) -> bool>;
pub type PropertyIdOp = Option<unsafe extern "C" fn(*mut JSContext, JSHandleObject, JSHandleObject,
                                                    Handle<*mut c_void>, JSMutableHandleValue) -> bool>;
pub type ElementIdOp = Option<unsafe extern "C" fn(*mut JSContext, JSHandleObject, JSHandleObject,
                                                   u32, JSMutableHandleValue) -> bool>;
pub type StrictGenericIdOp = Option<unsafe extern "C" fn(*mut JSContext, JSHandleObject, JSHandleId,
                                                         JSMutableHandleValue, bool) -> bool>;
pub type StrictPropertyIdOp = Option<unsafe extern "C" fn(*mut JSContext, JSHandleObject,
                                                          Handle<*mut c_void>, JSMutableHandleValue,
                                                          bool) -> bool>;
pub type StrictElementIdOp = Option<unsafe extern "C" fn(*mut JSContext, JSHandleObject, u32,
                                                         JSMutableHandleValue, bool) -> bool>;
pub type GenericAttributesOp = Option<unsafe extern "C" fn(*mut JSContext, JSHandleObject, JSHandleId,
                                                           *mut c_uint) -> bool>;
pub type PropertyAttributesOp = Option<unsafe extern "C" fn(*mut JSContext, JSHandleObject,
                                                            Handle<*mut c_void>, *mut uint) -> bool>;
pub type DeletePropertyOp = Option<unsafe extern "C" fn(*mut JSContext, JSHandleObject,
                                                        Handle<*mut c_void>, *mut bool) -> bool>;
pub type DeleteElementOp = Option<unsafe extern "C" fn(*mut JSContext, JSHandleObject, u32, *mut bool) -> bool>;
pub type WatchOp = Option<unsafe extern "C" fn(*mut JSContext, JSHandleObject, JSHandleId, JSHandleObject) -> bool>;
pub type UnwatchOp = Option<unsafe extern "C" fn(*mut JSContext, JSHandleObject, JSHandleId) -> bool>;
pub type SliceOp = Option<unsafe extern "C" fn(*mut JSContext, JSHandleObject, u32, u32, JSHandleObject) -> bool>;

#[repr(C)]
pub struct ObjectOps {
    pub lookupGeneric: LookupGenericOp,
    pub lookupProperty: LookupPropOp,
    pub lookupElement: LookupElementOp,
    pub defineGeneric: DefineGenericOp,
    pub defineProperty: DefinePropOp,
    pub defineElement: DefineElementOp,
    pub getGeneric: GenericIdOp,
    pub getProperty: PropertyIdOp,
    pub getElement: ElementIdOp,
    pub setGeneric: StrictGenericIdOp,
    pub setProperty: StrictPropertyIdOp,
    pub setElement: StrictElementIdOp,
    pub getGenericAttributes: GenericAttributesOp,
    pub setGenericAttributes: GenericAttributesOp,
    pub deleteProperty: DeletePropertyOp,
    pub deleteElement: DeleteElementOp,
    pub watch: WatchOp,
    pub unwatch: UnwatchOp,
    pub slice: SliceOp,

    pub enumerate: *const u8,
    pub thisObject: JSObjectOp,
}

pub enum ThingRootKind {
    THING_ROOT_OBJECT,
    THING_ROOT_SHAPE,
    THING_ROOT_BASE_SHAPE,
    THING_ROOT_TYPE_OBJECT,
    THING_ROOT_STRING,
    THING_ROOT_JIT_CODE,
    THING_ROOT_SCRIPT,
    THING_ROOT_LAZY_SCRIPT,
    THING_ROOT_ID,
    THING_ROOT_VALUE,
    THING_ROOT_TYPE,
    THING_ROOT_BINDINGS,
    THING_ROOT_PROPERTY_DESCRIPTOR,
    THING_ROOT_CUSTOM,
    THING_ROOT_LIMIT, //14
}

#[repr(C)]
pub struct ContextFriendFields {
    pub runtime_: *mut JSRuntime,
    pub compartment_: *mut libc::c_void,
    pub zone_: *mut libc::c_void,
    pub thingGCRooters: [*const *const libc::c_void, ..14], //THING_ROOT_LIMIT
    pub autoGCRooters: *mut libc::c_void,
}
