/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Rust wrappers around the raw JS apis

use libc::types::os::arch::c95::{size_t, c_uint};
use libc::c_char;
use std::ffi;
use std::str;
use std::ptr;
use std::mem;
use std::u32;
use std::default::Default;
use std::intrinsics::return_address;
use std::ops::{Deref, DerefMut};
use std::cell::UnsafeCell;
use jsapi::{JS_NewContext, JS_DestroyContext, JS_NewRuntime, JS_DestroyRuntime};
use jsapi::{JSContext, JSRuntime, JSObject, JSFlatString, JSFunction, JSString, Symbol, JSScript, jsid, Value};
use jsapi::{RuntimeOptionsRef, ContextOptionsRef, ReadOnlyCompileOptions};
use jsapi::{JS_SetErrorReporter, Evaluate3, JSErrorReport};
use jsapi::{JS_SetGCParameter, JSGCParamKey};
use jsapi::{Heap, Cell, HeapCellPostBarrier, HeapCellRelocate, HeapValuePostBarrier, HeapValueRelocate};
use jsapi::{ThingRootKind, ContextFriendFields};
use jsapi::{Rooted, RootedValue, Handle, MutableHandle};
use jsapi::{MutableHandleValue, HandleValue, HandleObject};
use jsapi::AutoObjectVector;
use jsapi::{ToBooleanSlow, ToNumberSlow, ToStringSlow};
use jsapi::{ToInt32Slow, ToUint32Slow, ToUint16Slow, ToInt64Slow, ToUint64Slow};
use jsapi::{JSAutoRequest, JS_BeginRequest, JS_EndRequest};
use jsapi::{JSAutoCompartment, JS_EnterCompartment, JS_LeaveCompartment};
use jsapi::{JSJitMethodCallArgs, JSJitGetterCallArgs, JSJitSetterCallArgs, CallArgs};
use jsapi::{JSVAL_NULL, JSVAL_VOID, JSID_VOID};
use jsval::UndefinedValue;
use glue::{CreateAutoObjectVector, AppendToAutoObjectVector, DeleteAutoObjectVector};
use glue::{NewCompileOptions, DeleteCompileOptions};
use default_stacksize;
use default_heapsize;

// ___________________________________________________________________________
// friendly Rustic API to runtimes

/// A wrapper for the `JSRuntime` and `JSContext` structures in SpiderMonkey.
pub struct Runtime {
    rt: *mut JSRuntime,
    cx: *mut JSContext,
}

impl Runtime {
    /// Creates a new `JSRuntime` and `JSContext`.
    pub fn new() -> Runtime {
        let js_runtime = unsafe {
            JS_NewRuntime(default_heapsize, ChunkSize as u32, ptr::null_mut())
        };
        assert!(!js_runtime.is_null());

        // Unconstrain the runtime's threshold on nominal heap size, to avoid
        // triggering GC too often if operating continuously near an arbitrary
        // finite threshold. This leaves the maximum-JS_malloc-bytes threshold
        // still in effect to cause periodical, and we hope hygienic,
        // last-ditch GCs from within the GC's allocator.
        unsafe {
            JS_SetGCParameter(js_runtime, JSGCParamKey::JSGC_MAX_BYTES, u32::MAX);
        }

        let js_context = unsafe {
            JS_NewContext(js_runtime, default_stacksize as size_t)
        };
        assert!(!js_context.is_null());

        unsafe {
            let runtimeopts = RuntimeOptionsRef(js_runtime);
            let contextopts = ContextOptionsRef(js_context);

            (*runtimeopts).set_varObjFix_(true);
            (*runtimeopts).set_baseline_(true);
            (*runtimeopts).set_ion_(true);

            (*contextopts).set_dontReportUncaught_(true);
            (*contextopts).set_autoJSAPIOwnsErrorReporting_(true);
            JS_SetErrorReporter(js_runtime, Some(reportError));
        }

        Runtime {
            rt: js_runtime,
            cx: js_context,
        }
    }

    /// Returns the `JSRuntime` object.
    pub fn rt(&self) -> *mut JSRuntime {
        self.rt
    }

    /// Returns the `JSContext` object.
    pub fn cx(&self) -> *mut JSContext {
        self.cx
    }

    pub fn evaluate_script(&self, glob: HandleObject, script: String, filename: String, line_num: u32)
                    -> Result<(),()> {
        let script_utf16: Vec<u16> = script.utf16_units().collect();
        let filename_cstr = ffi::CString::new(filename.as_bytes()).unwrap();
        debug!("Evaluating script from {} with content {}", filename, script);
        // SpiderMonkey does not approve of null pointers.
        let (ptr, len) = if script_utf16.len() == 0 {
            static empty: &'static [u16] = &[];
            (empty.as_ptr(), 0)
        } else {
            (script_utf16.as_ptr(), script_utf16.len() as c_uint)
        };
        assert!(!ptr.is_null());
        let _ar = JSAutoRequest::new(self.cx());
        let _ac = JSAutoCompartment::new(self.cx(), glob.get());
        let options = CompileOptionsWrapper::new(self.cx(), filename_cstr.as_ptr(), line_num);

        let scopechain = AutoObjectVectorWrapper::new(self.cx());

        let mut rval = RootedValue::new(self.cx(), UndefinedValue());

        unsafe {
            if 0 == Evaluate3(self.cx(), scopechain.ptr, options.ptr,
                              ptr as *const i16, len as size_t,
                              rval.handle_mut()) {
                debug!("...err!");
                Err(())
            } else {
                // we could return the script result but then we'd have
                // to root it and so forth and, really, who cares?
                debug!("...ok!");
                Ok(())
            }
        }
    }
}

impl Drop for Runtime {
    fn drop(&mut self) {
        unsafe {
            JS_DestroyContext(self.cx);
            JS_DestroyRuntime(self.rt);
        }
    }
}

// ___________________________________________________________________________
// Rooting API for standard JS things

trait RootKind {
    fn rootKind() -> ThingRootKind;
}

impl RootKind for *mut JSObject {
    fn rootKind() -> ThingRootKind { ThingRootKind::THING_ROOT_OBJECT }
}

impl RootKind for *mut JSFlatString {
    fn rootKind() -> ThingRootKind { ThingRootKind::THING_ROOT_STRING }
}

impl RootKind for *mut JSFunction {
    fn rootKind() -> ThingRootKind { ThingRootKind::THING_ROOT_OBJECT }
}

impl RootKind for *mut JSString {
    fn rootKind() -> ThingRootKind { ThingRootKind::THING_ROOT_STRING }
}

impl RootKind for *mut Symbol {
    fn rootKind() -> ThingRootKind { ThingRootKind::THING_ROOT_OBJECT }
}

impl RootKind for *mut JSScript {
    fn rootKind() -> ThingRootKind { ThingRootKind::THING_ROOT_SCRIPT }
}

impl RootKind for jsid {
    fn rootKind() -> ThingRootKind { ThingRootKind::THING_ROOT_ID }
}

impl RootKind for Value {
    fn rootKind() -> ThingRootKind { ThingRootKind::THING_ROOT_VALUE }
}

impl<T: RootKind> Rooted<T> {
    pub fn new_with_addr(cx: *mut JSContext, initial: T, addr: *const u8) -> Rooted<T> {
        let ctxfriend: &mut ContextFriendFields = unsafe {
            mem::transmute(cx)
        };

        let kind = T::rootKind() as usize;
        let root = Rooted::<T> {
            stack: &mut ctxfriend.thingGCRooters[kind],
            prev: ctxfriend.thingGCRooters[kind],
            ptr: initial,
        };

        ctxfriend.thingGCRooters[kind] = unsafe { mem::transmute(addr) };
        root
    }

    pub fn new(cx: *mut JSContext, initial: T) -> Rooted<T> {
        Rooted::new_with_addr(cx, initial, unsafe { return_address() })
    }

    pub fn handle(&self) -> Handle<T> {
        Handle::<T> {
            ptr: &self.ptr
        }
    }

    pub fn handle_mut(&mut self) -> MutableHandle<T> {
        MutableHandle::<T> {
            ptr: &mut self.ptr
        }
    }
}

impl<T: Copy> Handle<T> {
    pub fn get(&self) -> T {
        unsafe { *self.ptr }
    }
}

impl<T: Copy> Deref for Handle<T> {
    type Target = T;

    fn deref<'a>(&'a self) -> &'a T {
        unsafe { &*self.ptr }
    }
}

impl<T: Copy> Deref for MutableHandle<T> {
    type Target = T;

    fn deref<'a>(&'a self) -> &'a T {
        unsafe { &*self.ptr }
    }
}

impl<T: Copy> DerefMut for MutableHandle<T> {
    fn deref_mut<'a>(&'a mut self) -> &'a mut T {
        unsafe { &mut *self.ptr }
    }
}

impl HandleValue {
    pub fn null() -> HandleValue {
        HandleValue {
            ptr: &JSVAL_NULL
        }
    }

    pub fn undefined() -> HandleValue {
        HandleValue {
            ptr: &JSVAL_VOID
        }
    }
}

const ConstNullValue: *mut JSObject = 0 as *mut JSObject;

impl HandleObject {
    pub fn null() -> HandleObject {
        HandleObject { ptr: &ConstNullValue }
    }
}

impl<T: Copy> MutableHandle<T> {
    pub fn get(&self) -> T {
        unsafe { *self.ptr }
    }

    pub fn set(&self, v: T) {
        unsafe { *self.ptr = v }
    }

    pub fn handle(&self) -> Handle<T> {
        Handle { ptr: unsafe { &*self.ptr } }
    }
}

impl<T> Drop for Rooted<T> {
    fn drop(&mut self) {
        unsafe {
            assert!(*self.stack == mem::transmute(&*self));
            *self.stack = self.prev;
        }
    }
}

impl Default for jsid {
    fn default() -> jsid { JSID_VOID }
}

impl Default for Value {
    fn default() -> Value { UndefinedValue() }
}

const ChunkShift: usize = 20;
const ChunkSize: usize = 1 << ChunkShift;
const ChunkMask: usize = ChunkSize - 1;

#[cfg(target_pointer_width = "32")]
const ChunkLocationOffset: usize = ChunkSize - 2 * 4 - 8;

#[cfg(target_pointer_width = "64")]
const ChunkLocationOffset: usize = ChunkSize - 2 * 8 - 8;

const ChunkLocationBitNursery: usize = 1;

fn IsInsideNursery(p: *mut Cell) -> bool {
    if p.is_null() {
        return false;
    }

    let mut addr: usize = unsafe { mem::transmute(p) };
    addr = (addr & !ChunkMask) | ChunkLocationOffset;

    let location: *const u32 = unsafe { mem::transmute(addr) };
    let location = unsafe { *location };
    assert!(location != 0);
    (location & ChunkLocationBitNursery as u32) != 0
}

pub trait GCMethods<T> {
    fn needs_post_barrier(v: T) -> bool;
    unsafe fn post_barrier(v: *mut T);
    unsafe fn relocate(v: *mut T);
}

impl GCMethods<jsid> for jsid {
    fn needs_post_barrier(_: jsid) -> bool { return false; }
    unsafe fn post_barrier(_: *mut jsid) { unreachable!() }
    unsafe fn relocate(_: *mut jsid) { unreachable!() }
}

impl GCMethods<*mut JSObject> for *mut JSObject {
    fn needs_post_barrier(v: *mut JSObject) -> bool {
        return unsafe { IsInsideNursery(mem::transmute(v)) };
    }
    unsafe fn post_barrier(v: *mut *mut JSObject) {
        HeapCellPostBarrier(mem::transmute(v));
    }
    unsafe fn relocate(v: *mut *mut JSObject) {
        HeapCellRelocate(mem::transmute(v));
    }
}

impl GCMethods<*mut JSString> for *mut JSString {
    fn needs_post_barrier(v: *mut JSString) -> bool {
        return unsafe { IsInsideNursery(mem::transmute(v)) };
    }
    unsafe fn post_barrier(v: *mut *mut JSString) {
        HeapCellPostBarrier(mem::transmute(v));
    }
    unsafe fn relocate(v: *mut *mut JSString) {
        HeapCellRelocate(mem::transmute(v));
    }
}

impl GCMethods<*mut JSScript> for *mut JSScript {
    fn needs_post_barrier(v: *mut JSScript) -> bool {
        return unsafe { IsInsideNursery(mem::transmute(v)) };
    }
    unsafe fn post_barrier(v: *mut *mut JSScript) {
        HeapCellPostBarrier(mem::transmute(v));
    }
    unsafe fn relocate(v: *mut *mut JSScript) {
        HeapCellRelocate(mem::transmute(v));
    }
}

impl GCMethods<*mut JSFunction> for *mut JSFunction {
    fn needs_post_barrier(v: *mut JSFunction) -> bool {
        return unsafe { IsInsideNursery(mem::transmute(v)) };
    }
    unsafe fn post_barrier(v: *mut *mut JSFunction) {
        HeapCellPostBarrier(mem::transmute(v));
    }
    unsafe fn relocate(v: *mut *mut JSFunction) {
        HeapCellRelocate(mem::transmute(v));
    }
}

impl GCMethods<Value> for Value {
    fn needs_post_barrier(v: Value) -> bool {
        return v.is_object() &&
               unsafe { IsInsideNursery(mem::transmute(v.to_object())) };
    }
    unsafe fn post_barrier(v: *mut Value) {
        HeapValuePostBarrier(v);
    }
    unsafe fn relocate(v: *mut Value) {
        HeapValueRelocate(v);
    }
}

impl<T: GCMethods<T> + Copy> Heap<T> {
    pub fn set(&mut self, v: T) {
        unsafe {
            if T::needs_post_barrier(v) {
                *self.ptr.get() = v;
                T::post_barrier(self.ptr.get());
            } else if T::needs_post_barrier(self.get()) {
                T::relocate(self.ptr.get());
                *self.ptr.get() = v;
            } else {
                *self.ptr.get() = v;
            }
        }
    }

    pub fn get(&self) -> T {
        unsafe { *self.ptr.get() }
    }

    pub fn get_unsafe(&self) -> *mut T {
        self.ptr.get()
    }

    pub fn handle(&self) -> Handle<T> {
        Handle { ptr: self.ptr.get() as *const _ }
    }
}

impl Default for Heap<*mut JSObject> {
    fn default() -> Heap<*mut JSObject> {
        Heap {
            ptr: UnsafeCell::new(ptr::null_mut())
        }
    }
}

impl Default for Heap<Value> {
    fn default() -> Heap<Value> {
        Heap {
            ptr: UnsafeCell::new(Value::default())
        }
    }
}

impl<T: GCMethods<T> + Copy> Drop for Heap<T> {
    fn drop(&mut self) {
        if T::needs_post_barrier(self.get()) {
            unsafe { T::relocate(self.get_unsafe()) };
        }
    }
}


// ___________________________________________________________________________
// Implementations for various things in jsapi.rs

impl JSAutoRequest {
    pub fn new(cx: *mut JSContext) -> JSAutoRequest {
        unsafe { JS_BeginRequest(cx); }
        JSAutoRequest {
            mContext: cx
        }
    }
}

impl Drop for JSAutoRequest {
    fn drop(&mut self) {
        unsafe { JS_EndRequest(self.mContext); }
    }
}

impl JSAutoCompartment {
    pub fn new(cx: *mut JSContext, target: *mut JSObject) -> JSAutoCompartment {
        JSAutoCompartment {
            cx_: cx,
            oldCompartment_: unsafe { JS_EnterCompartment(cx, target) }
        }
    }
}

impl Drop for JSAutoCompartment {
    fn drop(&mut self) {
        unsafe { JS_LeaveCompartment(self.cx_, self.oldCompartment_); }
    }
}

impl JSJitMethodCallArgs {
    pub fn get(&self, i: u32) -> HandleValue {
        assert!(i < self.argc_);
        HandleValue {
            ptr: unsafe { self.argv_.offset(i as isize) }
        }
    }

    pub fn get_mut(&self, i: u32) -> MutableHandleValue {
        assert!(i < self.argc_);
        MutableHandleValue {
            ptr: unsafe { self.argv_.offset(i as isize) }
        }
    }

    pub fn rval(&self) -> MutableHandleValue {
        MutableHandleValue {
            ptr: unsafe { self.argv_.offset(-2) }
        }
    }
}

// XXX need to hack up bindgen to convert this better so we don't have
//     to duplicate so much code here
impl CallArgs {
    pub fn from_vp(vp: *mut Value, argc: u32) -> CallArgs {
        CallArgs {
            argv_: unsafe { vp.offset(2) },
            argc_: argc
        }
    }

    pub fn get(&self, i: u32) -> HandleValue {
        assert!(i < self.argc_);
        HandleValue {
            ptr: unsafe { self.argv_.offset(i as isize) }
        }
    }

    pub fn get_mut(&self, i: u32) -> MutableHandleValue {
        assert!(i < self.argc_);
        MutableHandleValue {
            ptr: unsafe { self.argv_.offset(i as isize) }
        }
    }

    pub fn rval(&self) -> MutableHandleValue {
        MutableHandleValue {
            ptr: unsafe { self.argv_.offset(-2) }
        }
    }

    pub fn thisv(&self) -> HandleValue {
        HandleValue {
            ptr: unsafe { self.argv_.offset(-1) }
        }
    }
}

impl JSJitGetterCallArgs {
    pub fn rval(&self) -> MutableHandleValue {
        self._base
    }
}

impl JSJitSetterCallArgs {
    pub fn get(&self, i: u32) -> HandleValue {
        assert!(i == 0);
        self._base
    }
}

// ___________________________________________________________________________
// Wrappers around things in jsglue.cpp

pub struct AutoObjectVectorWrapper {
    pub ptr: *mut AutoObjectVector
}

impl AutoObjectVectorWrapper {
    pub fn new(cx: *mut JSContext) -> AutoObjectVectorWrapper {
        AutoObjectVectorWrapper {
            ptr: unsafe {
                 CreateAutoObjectVector(cx)
            }
        }
    }

    pub fn append(&self, obj: *mut JSObject) -> bool {
        unsafe {
            AppendToAutoObjectVector(self.ptr, obj) != 0
        }
    }
}

impl Drop for AutoObjectVectorWrapper {
    fn drop(&mut self) {
        unsafe { DeleteAutoObjectVector(self.ptr) }
    }
}

pub struct CompileOptionsWrapper {
    pub ptr: *mut ReadOnlyCompileOptions
}

impl CompileOptionsWrapper {
    pub fn new(cx: *mut JSContext, file: *const i8, line: c_uint) -> CompileOptionsWrapper {
        CompileOptionsWrapper {
            ptr: unsafe { NewCompileOptions(cx, file, line) }
        }
    }
}

impl Drop for CompileOptionsWrapper {
    fn drop(&mut self) {
        unsafe { DeleteCompileOptions(self.ptr) }
    }
}

// ___________________________________________________________________________
// Fast inline converters

#[inline]
pub fn ToBoolean(v: HandleValue) -> bool {
    let val = unsafe { *v.ptr };

    if val.is_boolean() {
        return val.to_boolean();
    }

    if val.is_int32() {
        return val.to_int32() != 0;
    }

    if val.is_null_or_undefined() {
        return false;
    }

    if val.is_double() {
        let d = val.to_double();
        return !d.is_nan() && d != 0f64;
    }

    if val.is_symbol() {
        return true;
    }

    unsafe { ToBooleanSlow(v) != 0 }
}

#[inline]
pub fn ToNumber(cx: *mut JSContext, v: HandleValue) -> Result<f64, ()> {
    let val = unsafe { *v.ptr };
    if val.is_number() {
        return Ok(val.to_number());
    }

    let mut out = Default::default();
    unsafe {
        if ToNumberSlow(cx, val, &mut out) == 0 {
            Err(())
        } else {
            Ok(out)
        }
    }
}

#[inline]
fn convert_from_int32<T: Default + Copy>(
    cx: *mut JSContext,
    v: HandleValue,
    conv_fn: unsafe extern "C" fn(*mut JSContext, HandleValue, *mut T) -> u8)
        -> Result<T, ()> {

    let val = unsafe { *v.ptr };
    if val.is_int32() {
        let intval: i64 = val.to_int32() as i64;
        // TODO: do something better here that works on big endian
        let intval = unsafe { *(&intval as *const i64 as *const T) };
        return Ok(intval);
    }

    let mut out = Default::default();
    unsafe {
        if conv_fn(cx, v, &mut out) == 0 {
            Err(())
        } else {
            Ok(out)
        }
    }
}

#[inline]
pub fn ToInt32(cx: *mut JSContext, v: HandleValue) -> Result<i32, ()> {
    convert_from_int32::<i32>(cx, v, ToInt32Slow)
}

#[inline]
pub fn ToUint32(cx: *mut JSContext, v: HandleValue) -> Result<u32, ()> {
    convert_from_int32::<u32>(cx, v, ToUint32Slow)
}

#[inline]
pub fn ToUint16(cx: *mut JSContext, v: HandleValue) -> Result<u16, ()> {
    convert_from_int32::<u16>(cx, v, ToUint16Slow)
}

#[inline]
pub fn ToInt64(cx: *mut JSContext, v: HandleValue) -> Result<i64, ()> {
    convert_from_int32::<i64>(cx, v, ToInt64Slow)
}

#[inline]
pub fn ToUint64(cx: *mut JSContext, v: HandleValue) -> Result<u64, ()> {
    convert_from_int32::<u64>(cx, v, ToUint64Slow)
}

#[inline]
pub fn ToString(cx: *mut JSContext, v: HandleValue) -> *mut JSString {
    let val = unsafe { *v.ptr };
    if val.is_string() {
        return val.to_string();
    }

    unsafe {
        ToStringSlow(cx, v)
    }
}

pub unsafe extern fn reportError(_cx: *mut JSContext, msg: *const c_char, report: *mut JSErrorReport) {
    let fnptr = (*report).filename;
    let fname = if !fnptr.is_null() {
        let c_str = ffi::CStr::from_ptr(fnptr);
        str::from_utf8(c_str.to_bytes()).ok().unwrap().to_string()
    } else {
        "none".to_string()
    };
    let lineno = (*report).lineno;
    let c_str = ffi::CStr::from_ptr(msg);
    let msg = str::from_utf8(c_str.to_bytes()).ok().unwrap().to_string();
    error!("Error at {}:{}: {}\n", fname, lineno, msg);
}

#[cfg(test)]
pub mod test {
    use super::Runtime;

    #[test]
    pub fn dummy() {
        let _rt = Runtime::new();
    }

}
