/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/. */

use jsapi::{JSContext, JSPropertyDescriptor, jschar, JSRuntime};
use jsapi::{JSTracer, JSFunction, JSNative, JSErrorFormatString, JSFreeOp};
use jsapi::{JSClass, JSString, JSObject, jsid, JSVersion, JSTraceOp};
use jsapi::{Enum_OnNewGlobalHookOption, JSPrincipals, Enum_JSType, Struct_JSFreeOp};
use jsapi::{JSStrictPropertyOp, JSPropertyOp};
use jsapi::{Handle, MutableHandle};
use jsapi::{JSHandleObject, JSHandleId, JSMutableHandleValue};
use jsapi::{JSMutableHandleObject, JSHandleValue};

use jsfriendapi::JSJitInfo;
use jsval::JSVal;
use libc;
use libc::c_void;

pub static JS_STRUCTURED_CLONE_VERSION: u32 = 1;

pub type JSBool = libc::c_int;

pub struct ProxyTraps {
    pub preventExtensions: Option<unsafe extern "C" fn(*mut JSContext, JSHandleObject) -> bool>,
    pub getPropertyDescriptor: Option<unsafe extern "C" fn(*mut JSContext, JSHandleObject, JSHandleId, MutableHandle<JSPropertyDescriptor>, u32) -> bool>,
    pub getOwnPropertyDescriptor: Option<unsafe extern "C" fn(*mut JSContext, JSHandleObject, JSHandleId, MutableHandle<JSPropertyDescriptor>, u32) -> bool>,
    pub defineProperty: Option<unsafe extern "C" fn(*mut JSContext, JSHandleObject, JSHandleId, MutableHandle<JSPropertyDescriptor>) -> bool>,
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

pub fn CallJitSetterOp(info: *const JSJitInfo, cx: *mut JSContext, thisObj: JSHandleObject, specializedThis: *const libc::c_void, vp: *mut JSVal) -> c_bool;

pub fn CallJitGetterOp(info: *const JSJitInfo, cx: *mut JSContext, thisObj: JSHandleObject, specializedThis: *const libc::c_void, vp: *mut JSVal) -> c_bool;

pub fn CallJitMethodOp(info: *const JSJitInfo, cx: *mut JSContext, thisObj: JSHandleObject, specializedThis: *const libc::c_void, argc: libc::c_uint, vp: *mut JSVal) -> c_bool;

pub fn RUST_FUNCTION_VALUE_TO_JITINFO(v: JSVal) -> *const JSJitInfo;

pub fn SetFunctionNativeReserved(fun: *mut JSObject, which: libc::size_t, val: *JSVal);
pub fn GetFunctionNativeReserved(fun: *mut JSObject, which: libc::size_t) -> *JSVal;

pub fn CreateProxyHandler(traps: *const ProxyTraps, extra: *const libc::c_void) -> *const libc::c_void;
pub fn CreateWrapperProxyHandler(traps: *const ProxyTraps) -> *const libc::c_void;
pub fn NewProxyObject(cx: *mut JSContext, handler: *const libc::c_void, clasp: *const super::Class,
                      priv_: JSHandleValue, proto: *mut JSObject, parent: *mut JSObject) -> *mut JSObject;
pub fn WrapperNew(cx: *mut JSContext, obj: JSHandleObject, parent: JSHandleObject,
                  handler: *const libc::c_void, clasp: *const super::Class, singleton: bool) -> *mut JSObject;

pub fn GetProxyExtra(obj: *mut JSObject, slot: libc::c_uint) -> JSVal;
pub fn GetProxyPrivate(obj: *mut JSObject) -> JSVal;
pub fn SetProxyExtra(obj: *mut JSObject, slot: libc::c_uint, val: JSVal);

pub fn GetObjectProto(cx: *mut JSContext, obj: JSHandleObject, proto: JSMutableHandleObject) -> c_bool;
pub fn GetObjectParent(obj: *mut JSObject) -> *mut JSObject;

pub fn RUST_JSID_IS_INT(id: jsid) -> c_bool;
pub fn RUST_JSID_TO_INT(id: jsid) -> libc::c_int;
pub fn RUST_JSID_IS_STRING(id: jsid) -> c_bool;
pub fn RUST_JSID_TO_STRING(id: jsid) -> *mut JSString;

pub fn RUST_SET_JITINFO(func: *mut JSFunction, info: *const JSJitInfo);

pub fn RUST_INTERNED_STRING_TO_JSID(cx: *mut JSContext, str: *mut JSString) -> jsid;

pub fn DefineFunctionWithReserved(cx: *mut JSContext, obj: *mut JSObject,
                                  name: *const libc::c_char, call: JSNative, nargs: libc::c_uint,
                                  attrs: libc::c_uint) -> *mut JSObject;
pub fn GetObjectJSClass(obj: *mut JSObject) -> *const JSClass;
pub fn RUST_js_GetErrorMessage(userRef: *mut libc::c_void, locale: *const libc::c_char,
                               errorNumber: libc::c_uint) -> *const JSErrorFormatString;
pub fn IsProxyHandlerFamily(obj: JSHandleObject) -> bool;
pub fn GetProxyHandlerExtra(obj: JSHandleObject) -> *const libc::c_void;
pub fn GetProxyHandler(obj: JSHandleObject) -> *mut libc::c_void;
pub fn InvokeGetOwnPropertyDescriptor(handler: *mut libc::c_void, cx: *mut JSContext, proxy: JSHandleObject, id: JSHandleId, desc: MutableHandle<JSPropertyDescriptor>, flags: libc::c_uint) -> bool;
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
                         argc: libc::size_t, argv: *JSVal, rval: JSMutableHandleValue) -> bool;
pub fn CompileUCFunction(cx: *mut JSContext, obj: JSHandleObject,
                         name: *libc::c_schar, nargs: libc::c_uint,
                         argnames: **libc::c_schar, chars: *jschar,
                         length: libc::size_t, url: *libc::c_schar,
                         lineno: libc::c_uint) -> *mut c_void;
pub fn CompileEventHandler(cx: *mut JSContext, name: *libc::c_char,
                           nargs: libc::c_uint, argnames: **libc::c_char,
                           chars: *u16, length: libc::size_t,
                           url: *libc::c_char, lineNo: libc::c_uint) -> *mut JSObject;

pub fn proxy_LookupGeneric(cx: *mut JSContext, obj: JSHandleObject, id: JSHandleId,
                           objp: JSMutableHandleObject, propp: MutableHandle<*mut c_void>) -> bool;
pub fn proxy_LookupProperty(cx: *mut JSContext, obj: JSHandleObject, name: Handle<*mut c_void>,
                            objp: JSMutableHandleObject, propp: MutableHandle<*mut c_void>) -> bool;
pub fn proxy_LookupElement(cx: *mut JSContext, obj: JSHandleObject, index: u32,
                           objp: JSMutableHandleObject, propp: MutableHandle<*mut c_void>) -> bool;
pub fn proxy_DefineGeneric(cx: *mut JSContext, obj: JSHandleObject, id: JSHandleId,
                           value: JSHandleValue, getter: JSPropertyOp,
                           setter: JSStrictPropertyOp, attrs: uint) -> bool;
pub fn proxy_DefineProperty(cx: *mut JSContext, obj: JSHandleObject, name: Handle<*mut c_void>,
                           value: JSHandleValue, getter: JSPropertyOp,
                           setter: JSStrictPropertyOp, attrs: uint) -> bool;
pub fn proxy_DefineElement(cx: *mut JSContext, obj: JSHandleObject, index: u32,
                           value: JSHandleValue, getter: JSPropertyOp,
                           setter: JSStrictPropertyOp, attrs: uint) -> bool;
pub fn proxy_GetGeneric(cx: *mut JSContext, obj: JSHandleObject, receiver: JSHandleObject,
                        id: JSHandleId, vp: JSMutableHandleValue) -> bool;
pub fn proxy_GetProperty(cx: *mut JSContext, obj: JSHandleObject, receiver: JSHandleObject,
                        name: Handle<*mut c_void>, vp: JSMutableHandleValue) -> bool;
pub fn proxy_GetElement(cx: *mut JSContext, obj: JSHandleObject, receiver: JSHandleObject,
                        index: u32, vp: JSMutableHandleValue) -> bool;
pub fn proxy_SetGeneric(cx: *mut JSContext, obj: JSHandleObject, id: JSHandleId,
                        bp: JSMutableHandleValue, strict: bool) -> bool;
pub fn proxy_SetProperty(cx: *mut JSContext, obj: JSHandleObject, name: Handle<*mut c_void>,
                         bp: JSMutableHandleValue, strict: bool) -> bool;
pub fn proxy_SetElement(cx: *mut JSContext, obj: JSHandleObject, index: u32,
                        vp: JSMutableHandleValue, strict: bool) -> bool;
pub fn proxy_GetGenericAttributes(cx: *mut JSContext, obj: JSHandleObject, id: JSHandleId,
                                  attrsp: *mut uint) -> bool;
pub fn proxy_SetGenericAttributes(cx: *mut JSContext, obj: JSHandleObject, id: JSHandleId,
                                  attrsp: *mut uint) -> bool;
pub fn proxy_DeleteProperty(cx: *mut JSContext, obj: JSHandleObject, name: Handle<*mut c_void>,
                            succeeded: *mut bool) -> bool;
pub fn proxy_DeleteElement(cx: *mut JSContext, obj: JSHandleObject, index: u32,
                           succeeded: *mut bool) -> bool;
pub fn proxy_Trace(cx: *mut JSTracer, obj: *mut JSObject);
pub fn proxy_WeakmapKeyDelegate(obj: *mut JSObject) -> *mut JSObject;
pub fn proxy_Convert(cx: *mut JSContext, obj: JSHandleObject, hint: Enum_JSType,
                     vp: JSMutableHandleValue) -> c_bool;
pub fn proxy_Finalize(fop: *mut Struct_JSFreeOp, obj: *mut JSObject);
pub fn proxy_HasInstance(cx: *mut JSContext, proxy: JSHandleObject, v: JSMutableHandleValue,
                         bp: *mut c_bool) -> c_bool;
pub fn proxy_Call(cx: *mut JSContext, argc: uint, vp: *mut JSVal) -> bool;
pub fn proxy_Construct(cx: *mut JSContext, argc: uint, vp: *mut JSVal) -> bool;
pub fn proxy_innerObject(cx: *mut JSContext, obj: JSHandleObject) -> *mut JSObject;
pub fn proxy_Watch(cx: *mut JSContext, obj: JSHandleObject, id: JSHandleId,
                   callable: JSHandleObject) -> bool;
pub fn proxy_Unwatch(cx: *mut JSContext, obj: JSHandleObject, id: JSHandleId) -> bool;
pub fn proxy_Slice(cx: *mut JSContext, obj: JSHandleObject, begin: u32, end: u32,
                   result: JSHandleObject) -> bool;

pub fn objectNeedsPostBarrier(obj: *mut JSObject) -> int;
pub fn objectPostBarrier(obj: *mut *mut JSObject);
pub fn objectRelocate(obj: *mut *mut JSObject);

pub fn getPersistentRootedObjectList(rt: *mut JSRuntime) -> *libc::c_void;
pub fn insertObjectLinkedListElement(list: *libc::c_void, elem: *libc::c_void);
}
