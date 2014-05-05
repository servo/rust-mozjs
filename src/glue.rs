/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/. */

use jsapi::{JSContext, JSHandleObject, JSHandleId, JSPropertyDescriptor, JSMutableHandleValue};
use jsapi::{JSTracer, JSFunction, JSNative, JSErrorFormatString, JSFreeOp, JSMutableHandleObject};
use jsapi::{JSClass, JSString, JSObject, jsid, JSVersion, JSHandleValue, JSTraceOp};
use jsapi::{Enum_OnNewGlobalHookOption, JSPrincipals, Enum_JSType};
use jsfriendapi::JSJitInfo;
use jsval::JSVal;
use libc;

pub static JS_STRUCTURED_CLONE_VERSION: u32 = 1;

pub type JSBool = libc::c_int;

pub struct JSMutableHandle<T> {
    pub unnamed_field1: *mut *mut T,
}

pub struct ProxyTraps {
    pub preventExtensions: Option<unsafe extern "C" fn(*mut JSContext, JSHandleObject) -> bool>,
    pub getPropertyDescriptor: Option<unsafe extern "C" fn(*mut JSContext, JSHandleObject, JSHandleId, JSMutableHandle<JSPropertyDescriptor>, u32) -> bool>,
    pub getOwnPropertyDescriptor: Option<unsafe extern "C" fn(*mut JSContext, JSHandleObject, JSHandleId, JSMutableHandle<JSPropertyDescriptor>, u32) -> bool>,
    pub defineProperty: Option<unsafe extern "C" fn(*mut JSContext, JSHandleObject, JSHandleId, JSMutableHandle<JSPropertyDescriptor>) -> bool>,
    pub getOwnPropertyNames: *const u8, //XXX need a representation for AutoIdVector&
    pub delete_: Option<unsafe extern "C" fn(*mut JSContext, JSHandleObject, JSHandleId, *mut bool) -> JSBool>,
    pub enumerate: *const u8, //XXX need a representation for AutoIdVector&

    pub has: Option<unsafe extern "C" fn(*mut JSContext, JSHandleObject, JSHandleId, *mut JSBool) -> JSBool>,
    pub hasOwn: Option<unsafe extern "C" fn(*mut JSContext, JSHandleObject, JSHandleId, *mut bool) -> bool>,
    pub get: Option<unsafe extern "C" fn(*mut JSContext, JSHandleObject, JSHandleObject, JSHandleId, JSMutableHandleValue) -> bool>,
    pub set: Option<unsafe extern "C" fn(*mut JSContext, JSHandleObject, JSHandleObject, JSHandleId, JSBool, JSMutableHandleValue) -> JSBool>,
    pub keys: *const u8, //XXX need a representation for AutoIdVector&
    pub iterate: Option<unsafe extern "C" fn(*mut JSContext, JSHandleObject, uint, JSMutableHandleValue) -> JSBool>,

    pub isExtensible: Option<unsafe extern "C" fn(*mut JSContext, JSHandleObject, *mut bool) -> bool>,
    pub call: Option<unsafe extern "C" fn(*mut JSContext, JSHandleObject, uint, JSMutableHandleValue) -> JSBool>,
    pub construct: Option<unsafe extern "C" fn(*mut JSContext, JSHandleObject, uint, JSMutableHandleValue, JSMutableHandleValue) -> JSBool>,
    pub nativeCall: *const u8, //XXX need a representation for IsAcceptableThis, NativeImpl, and CallArgs
    pub hasInstance: Option<unsafe extern "C" fn(*mut JSContext, JSHandleObject, JSMutableHandleValue, *mut JSBool) -> JSBool>,
    pub objectClassIs: Option<unsafe extern "C" fn(JSHandleObject, uint, *mut JSContext) -> JSBool>, //XXX ESClassValue enum
    pub fun_toString: Option<unsafe extern "C" fn(*mut JSContext, JSHandleObject, uint) -> *JSString>,
    //regexp_toShared: *u8,
    pub defaultValue: Option<unsafe extern "C" fn(*mut JSContext, JSHandleObject, Enum_JSType, JSMutableHandleValue) -> JSBool>, //XXX JSType enum
    pub finalize: Option<unsafe extern "C" fn(*mut JSFreeOp, *mut JSObject)>,
    pub getPrototypeOf: Option<unsafe extern "C" fn(*mut JSContext, JSHandleObject, JSMutableHandleObject) -> bool>,
    pub trace: Option<unsafe extern "C" fn(*mut JSTracer, *mut JSObject)>
}

#[link(name = "jsglue")]
extern { }

#[cfg(target_os = "android")]
#[link(name = "stdc++")]
extern { }

#[cfg(target_os = "android")]
#[link(name = "gcc")]
extern { }

extern {

pub fn RUST_JS_NumberValue(d: f64) -> JSVal;

pub fn CallJitSetterOp(info: *const JSJitInfo, cx: *mut JSContext, thisObj: JSHandleObject, specializedThis: *mut libc::c_void, vp: *mut JSVal) -> c_bool;

pub fn CallJitGetterOp(info: *const JSJitInfo, cx: *mut JSContext, thisObj: JSHandleObject, specializedThis: *mut libc::c_void, vp: *mut JSVal) -> c_bool;

pub fn CallJitMethodOp(info: *const JSJitInfo, cx: *mut JSContext, thisObj: JSHandleObject, specializedThis: *mut libc::c_void, argc: libc::c_uint, vp: *mut JSVal) -> c_bool;

pub fn RUST_FUNCTION_VALUE_TO_JITINFO(v: JSVal) -> *const JSJitInfo;

pub fn SetFunctionNativeReserved(fun: JSHandleObject, which: libc::size_t, val: *JSVal);
pub fn GetFunctionNativeReserved(fun: JSHandleObject, which: libc::size_t) -> *JSVal;

pub fn CreateProxyHandler(traps: *const ProxyTraps, extra: *const libc::c_void) -> *const libc::c_void;
pub fn CreateWrapperProxyHandler(traps: *const ProxyTraps) -> *const libc::c_void;
pub fn NewProxyObject(cx: *mut JSContext, handler: *const libc::c_void, priv_: *const JSVal,
                      proto: JSHandleObject, parent: JSHandleObject, call: JSHandleObject,
                      construct: JSHandleObject) -> *mut JSObject;
pub fn WrapperNew(cx: *mut JSContext, obj: JSHandleObject, parent: JSHandleObject, handler: *const libc::c_void) -> *mut JSObject;

pub fn GetProxyExtra(obj: JSHandleObject, slot: libc::c_uint) -> JSVal;
pub fn GetProxyPrivate(obj: JSHandleObject) -> JSVal;
pub fn SetProxyExtra(obj: JSHandleObject, slot: libc::c_uint, val: JSVal);

pub fn GetObjectProto(cx: *mut JSContext, obj: JSHandleObject, proto: JSMutableHandleObject) -> c_bool;
pub fn GetObjectParent(obj: *mut JSObject) -> *mut JSObject;

pub fn RUST_JSID_IS_INT(id: JSHandleId) -> c_bool;
pub fn RUST_JSID_TO_INT(id: JSHandleId) -> libc::c_int;
pub fn RUST_JSID_IS_STRING(id: JSHandleId) -> c_bool;
pub fn RUST_JSID_TO_STRING(id: JSHandleId) -> *mut JSString;

pub fn RUST_SET_JITINFO(func: *mut JSFunction, info: *const JSJitInfo);

pub fn RUST_INTERNED_STRING_TO_JSID(cx: *mut JSContext, str: *mut JSString) -> jsid;

pub fn DefineFunctionWithReserved(cx: *mut JSContext, obj: JSHandleObject,
                                  name: *const libc::c_char, call: JSNative, nargs: libc::c_uint,
                                  attrs: libc::c_uint) -> *mut JSObject;
pub fn GetObjectJSClass(obj: JSHandleObject) -> *const JSClass;
pub fn RUST_js_GetErrorMessage(userRef: *mut libc::c_void, locale: *const libc::c_char,
                               errorNumber: libc::c_uint) -> *const JSErrorFormatString;
pub fn IsProxyHandlerFamily(obj: JSHandleObject) -> bool;
pub fn GetProxyHandlerExtra(obj: JSHandleObject) -> *const libc::c_void;
pub fn GetProxyHandler(obj: JSHandleObject) -> *mut libc::c_void;
pub fn InvokeGetOwnPropertyDescriptor(handler: *mut libc::c_void, cx: *mut JSContext, proxy: JSHandleObject, id: JSHandleId, desc: JSMutableHandle<JSPropertyDescriptor>, flags: libc::c_uint) -> booll;
pub fn GetGlobalForObjectCrossCompartment(obj: *mut JSObject) -> *mut JSObject;
pub fn ReportError(cx: *mut JSContext, error: *const libc::c_char);
pub fn IsWrapper(obj: *mut JSObject) -> JSBool;
pub fn UnwrapObject(obj: *mut JSObject, stopAtOuter: bool) -> *mut JSObject;

pub fn ContextOptions_SetVarObjFix(cx: *mut JSContext, enable: bool);
pub fn CompartmentOptions_SetVersion(cx: *mut JSContext, version: JSVersion);
pub fn CompartmentOptions_SetTraceGlobal(cx: *mut JSContext, op: JSTraceOp);

pub fn ToBoolean(v: JSHandleValue) -> bool;
pub fn ToString(cx: *mut JSContext, v: JSHandleValue) -> *mut JSString;
pub fn ToNumber(cx: *mut JSContext, v: JSHandleValue, out: *mut f64) -> bool;
pub fn ToUint16(cx: *mut JSContext, v: JSHandleValue, out: *mut u16) -> bool;
pub fn ToInt32(cx: *mut JSContext, v: JSHandleValue, out: *mut i32) -> bool;
pub fn ToUint32(cx: *mut JSContext, v: JSHandleValue, out: *mut u32) -> bool;
pub fn ToInt64(cx: *mut JSContext, v: JSHandleValue, out: *mut i64) -> bool;
pub fn ToUint64(cx: *mut JSContext, v: JSHandleValue, out: *mut u64) -> bool;

//XXX Heap pub fn AddObjectRoot(cx: *mut JSContext, obj: *mut *mut JSObject) -> bool;
//XXX Heap pub fn RemoveObjectRoot(cx: *mut JSContext, obj: *mut *mut libc::c_void);

pub fn NewGlobalObject(cx: *mut JSContext, clasp: *const JSClass,
                       principals: *mut JSPrincipals,
                       hookOption: Enum_OnNewGlobalHookOption) -> *mut JSObject;

pub fn CallFunctionValue(cx: *mut JSContext, obj: JSHandleObject, fval: JSHandleValue,
                         rval: JSMutableHandleValue) -> bool;
}
