/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/. */

use jsapi::{JSContext, JSObject, JSPropertyDescriptor, JSMutableHandleValue, JSHandleValue};
use jsapi::Struct_JSStructuredCloneCallbacks;
use glue::JSBool;
use libc;

pub type JSJitPropertyOp = *const u8;

pub enum OpType {
    Getter = 0,
    Setter = 1,
    Method = 2,
}

pub struct JSJitInfo {
    pub op: JSJitPropertyOp,
    pub protoID: u16,
    pub depth: u16,
    pub type_and_aliasSet: u8,
    pub returnType: u8,
    pub infallible_and_isMovable_and_isInSlot_and_isTypedMethod_and_slotIndex: u16,
}

extern {
pub fn JS_ObjectToOuterObject(cx: *mut JSContext,
                              obj: *mut JSObject) -> *mut JSObject;
pub fn JS_WrapPropertyDescriptor(cx: *mut JSContext,
                                 desc: *mut JSPropertyDescriptor) -> JSBool;
pub fn JS_ReadStructuredClone(cx: *mut JSContext, data: *const u64, nbytes: libc::size_t,
                              version: u32, vp: JSMutableHandleValue,
                              optionalCallbacks: *const Struct_JSStructuredCloneCallbacks,
                              closure: *mut libc::c_void) -> bool;
pub fn JS_WriteStructuredClone(cx: *mut JSContext, v: JSHandleValue, datap: *mut *mut u64,
                               nbytesp: *mut libc::size_t,
                               optionalCallbacks: *const Struct_JSStructuredCloneCallbacks,
                               closure: *mut libc::c_void,
                               transferable: JSHandleValue) -> bool;
}

//pub type JSJitInfo = JSJitInfo_struct;

pub mod bindgen {
    use jsapi::{JSContext, JSObject, JSClass, JSRuntime, JSHandleObject};
    use libc::uintptr_t;

    extern {
        pub fn JS_NewObjectWithUniqueType(cx: *mut JSContext, clasp: *const JSClass,
                                          proto: JSHandleObject, parent: JSHandleObject) -> *mut JSObject;
        pub fn JS_GetAddressableObject(rt: *mut JSRuntime, candidateObj: uintptr_t) -> *mut JSObject;
    }
}
