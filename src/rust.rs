/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Rust wrappers around the raw JS apis


use libc::{size_t, c_uint, c_char};

use std::char;
use std::ffi;
use std::ptr;
use std::slice;
use std::mem;
use std::u32;
use std::default::Default;
use std::ops::{Deref, DerefMut};
use std::cell::{Cell, UnsafeCell};
use std::marker::PhantomData;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use consts::{JSCLASS_RESERVED_SLOTS_MASK, JSCLASS_GLOBAL_SLOT_COUNT};
use consts::{JSCLASS_IS_DOMJSCLASS, JSCLASS_IS_GLOBAL};

use conversions::jsstr_to_string;

use jsapi;
use jsapi::{AutoGCRooter, AutoIdVector, AutoObjectVector, CallArgs, CompartmentOptions, ContextFriendFields};
use jsapi::{Evaluate2, HandleBase, HandleValueArray, Heap};
use jsapi::{HeapObjectPostBarrier, HeapValuePostBarrier, InitSelfHostedCode, IsWindowSlow, JS_BeginRequest};
use jsapi::{JS_DefineFunctions, JS_DefineProperties, JS_DestroyRuntime, JS_EndRequest, JS_ShutDown};
use jsapi::{JS_EnterCompartment, JS_EnumerateStandardClasses, JS_GetContext, JS_GlobalObjectTraceHook};
use jsapi::{JS_Init, JS_LeaveCompartment, JS_MayResolveStandardClass, JS_NewRuntime, JS_ResolveStandardClass};
use jsapi::{JS_SetGCParameter, JS_SetNativeStackQuota, JS_WrapValue, JSAutoCompartment};
use jsapi::{JSClass, JSCLASS_RESERVED_SLOTS_SHIFT, JSClassOps, JSCompartment, JSContext};
use jsapi::{JSErrorReport, JSFlatString, JSFunction, JSFunctionSpec, JSGCParamKey};
use jsapi::{JSID_VOID, JSJitGetterCallArgs, JSJitMethodCallArgs, JSJitSetterCallArgs};
use jsapi::{JSNativeWrapper, JSObject, JSPropertySpec, JSRuntime, JSScript};
use jsapi::{JSString, JSTracer, MutableHandleBase};
use jsapi::{NullHandleValue, Object, ObjectGroup,ReadOnlyCompileOptions, Rooted};
use jsapi::{RootedBase, RuntimeOptionsRef, SetWarningReporter, Symbol, ToBooleanSlow};
use jsapi::{ToInt32Slow, ToInt64Slow, ToNumberSlow, ToStringSlow, ToUint16Slow};
use jsapi::{ToUint32Slow, ToUint64Slow, ToWindowProxyIfWindow, UndefinedHandleValue};
use jsapi::{Value, jsid, PerThreadDataFriendFields, PerThreadDataFriendFields_RuntimeDummy};
use jsapi::{CaptureCurrentStack, BuildStackString, IsSavedFrame};
use jsapi::{AutoGCRooter_jspubtd_h_unnamed_1 as AutoGCRooterTag, _vftable_CustomAutoRooter as CustomAutoRooterVFTable};
use jsapi::Handle as RawHandle;
use jsapi::MutableHandle as RawMutableHandle;
use jsapi::HandleValue as RawHandleValue;

use jsval::{ObjectValue, UndefinedValue};

use glue::{AppendToAutoObjectVector, CallFunctionTracer, CallIdTracer, CallObjectTracer};
use glue::{CallScriptTracer, CallStringTracer, CallValueTracer, CreateAutoIdVector};
use glue::{CreateAutoObjectVector, CreateCallArgsFromVp, DeleteAutoObjectVector};
use glue::{DestroyAutoIdVector, DeleteCompileOptions, NewCompileOptions, SliceAutoIdVector};
use glue::{CallObjectRootTracer, CallValueRootTracer};

use panic::maybe_resume_unwind;

use default_heapsize;

// From Gecko:
// Our "default" stack is what we use in configurations where we don't have a compelling reason to
// do things differently. This is effectively 1MB on 64-bit platforms.
const STACK_QUOTA: usize = 128 * 8 * 1024;

// From Gecko:
// The JS engine permits us to set different stack limits for system code,
// trusted script, and untrusted script. We have tests that ensure that
// we can always execute 10 "heavy" (eval+with) stack frames deeper in
// privileged code. Our stack sizes vary greatly in different configurations,
// so satisfying those tests requires some care. Manual measurements of the
// number of heavy stack frames achievable gives us the following rough data,
// ordered by the effective categories in which they are grouped in the
// JS_SetNativeStackQuota call (which predates this analysis).
//
// (NB: These numbers may have drifted recently - see bug 938429)
// OSX 64-bit Debug: 7MB stack, 636 stack frames => ~11.3k per stack frame
// OSX64 Opt: 7MB stack, 2440 stack frames => ~3k per stack frame
//
// Linux 32-bit Debug: 2MB stack, 426 stack frames => ~4.8k per stack frame
// Linux 64-bit Debug: 4MB stack, 455 stack frames => ~9.0k per stack frame
//
// Windows (Opt+Debug): 900K stack, 235 stack frames => ~3.4k per stack frame
//
// Linux 32-bit Opt: 1MB stack, 272 stack frames => ~3.8k per stack frame
// Linux 64-bit Opt: 2MB stack, 316 stack frames => ~6.5k per stack frame
//
// We tune the trusted/untrusted quotas for each configuration to achieve our
// invariants while attempting to minimize overhead. In contrast, our buffer
// between system code and trusted script is a very unscientific 10k.
const SYSTEM_CODE_BUFFER: usize = 10 * 1024;

// Gecko's value on 64-bit.
const TRUSTED_SCRIPT_BUFFER: usize = 8 * 12800;

trait ToResult {
    fn to_result(self) -> Result<(), ()>;
}

impl ToResult for bool {
    fn to_result(self) -> Result<(), ()> {
        if self {
            Ok(())
        } else {
            Err(())
        }
    }
}

// ___________________________________________________________________________
// friendly Rustic API to runtimes

thread_local!(static CONTEXT: Cell<*mut JSContext> = Cell::new(ptr::null_mut()));

lazy_static! {
    static ref OUTSTANDING_RUNTIMES: AtomicUsize = AtomicUsize::new(0);
    static ref SHUT_DOWN: AtomicBool = AtomicBool::new(false);
}

/// A wrapper for the `JSRuntime` and `JSContext` structures in SpiderMonkey.
pub struct Runtime {
    rt: *mut JSRuntime,
    cx: *mut JSContext,
}

impl Runtime {
    /// Get the `JSContext` for this thread.
    pub fn get() -> *mut JSContext {
        let cx = CONTEXT.with(|context| {
            context.get()
        });
        assert!(!cx.is_null());
        cx
    }

    /// Creates a new `JSRuntime` and `JSContext`.
    pub fn new() -> Result<Runtime, ()> {
        if SHUT_DOWN.load(Ordering::SeqCst) {
            return Err(());
        }

        OUTSTANDING_RUNTIMES.fetch_add(1, Ordering::SeqCst);

        unsafe {
            struct TopRuntime(*mut JSRuntime);
            unsafe impl Sync for TopRuntime {}

            lazy_static! {
                static ref PARENT: TopRuntime = {
                    unsafe {
                        assert!(JS_Init());
                        let runtime = JS_NewRuntime(
                            default_heapsize, ChunkSize as u32, ptr::null_mut());
                        assert!(!runtime.is_null());
                        let context = JS_GetContext(runtime);
                        assert!(!context.is_null());
                        InitSelfHostedCode(context);
                        TopRuntime(runtime)
                    }
                };
            }

            let js_runtime =
                JS_NewRuntime(default_heapsize, ChunkSize as u32, PARENT.0);
            assert!(!js_runtime.is_null());

            // Unconstrain the runtime's threshold on nominal heap size, to avoid
            // triggering GC too often if operating continuously near an arbitrary
            // finite threshold. This leaves the maximum-JS_malloc-bytes threshold
            // still in effect to cause periodical, and we hope hygienic,
            // last-ditch GCs from within the GC's allocator.
            JS_SetGCParameter(
                js_runtime, JSGCParamKey::JSGC_MAX_BYTES, u32::MAX);

            JS_SetNativeStackQuota(
                js_runtime,
                STACK_QUOTA,
                STACK_QUOTA - SYSTEM_CODE_BUFFER,
                STACK_QUOTA - SYSTEM_CODE_BUFFER - TRUSTED_SCRIPT_BUFFER);

            let js_context = JS_GetContext(js_runtime);
            assert!(!js_context.is_null());

            CONTEXT.with(|context| {
                assert!(context.get().is_null());
                context.set(js_context);
            });

            InitSelfHostedCode(js_context);

            let runtimeopts = RuntimeOptionsRef(js_runtime);
            (*runtimeopts).set_baseline_(true);
            (*runtimeopts).set_ion_(true);
            (*runtimeopts).set_nativeRegExp_(true);

            SetWarningReporter(js_runtime, Some(report_warning));

            JS_BeginRequest(js_context);

            Ok(Runtime {
                rt: js_runtime,
                cx: js_context,
            })
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

    pub fn evaluate_script(&self, glob: HandleObject, script: &str, filename: &str,
                           line_num: u32, rval: MutableHandleValue)
                    -> Result<(),()> {
        let script_utf16: Vec<u16> = script.encode_utf16().collect();
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
        let _ac = JSAutoCompartment::new(self.cx(), glob.get());
        let options = CompileOptionsWrapper::new(self.cx(), filename_cstr.as_ptr(), line_num);

        unsafe {
            if !Evaluate2(self.cx(), options.ptr, ptr as *const u16, len as size_t, rval.into()) {
                debug!("...err!");
                maybe_resume_unwind();
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
            JS_EndRequest(self.cx);
            JS_DestroyRuntime(self.rt);

            CONTEXT.with(|context| {
                assert_eq!(context.get(), self.cx);
                context.set(ptr::null_mut());
            });

            if OUTSTANDING_RUNTIMES.fetch_sub(1, Ordering::SeqCst) == 1 {
                SHUT_DOWN.store(true, Ordering::SeqCst);
                JS_ShutDown();
            }
        }
    }
}

// ___________________________________________________________________________
// Rooting API for standard JS things

pub trait RootKind {
    #[inline(always)]
    fn rootKind() -> jsapi::RootKind;
}

impl RootKind for *mut JSObject {
    #[inline(always)]
    fn rootKind() -> jsapi::RootKind { jsapi::RootKind::Object }
}

impl RootKind for *mut JSFlatString {
    #[inline(always)]
    fn rootKind() -> jsapi::RootKind { jsapi::RootKind::String }
}

impl RootKind for *mut JSFunction {
    #[inline(always)]
    fn rootKind() -> jsapi::RootKind { jsapi::RootKind::Object }
}

impl RootKind for *mut JSString {
    #[inline(always)]
    fn rootKind() -> jsapi::RootKind { jsapi::RootKind::String }
}

impl RootKind for *mut Symbol {
    #[inline(always)]
    fn rootKind() -> jsapi::RootKind { jsapi::RootKind::Symbol }
}

impl RootKind for *mut JSScript {
    #[inline(always)]
    fn rootKind() -> jsapi::RootKind { jsapi::RootKind::Script }
}

impl RootKind for jsid {
    #[inline(always)]
    fn rootKind() -> jsapi::RootKind { jsapi::RootKind::Id }
}

impl RootKind for Value {
    #[inline(always)]
    fn rootKind() -> jsapi::RootKind { jsapi::RootKind::Value }
}

// Creates a C string literal `$str`.
macro_rules! c_str {
    ($str:expr) => {
        concat!($str, "\0").as_ptr() as *const ::std::os::raw::c_char
    }
}

/// Types that can be traced.
///
/// This trait is unsafe; if it is implemented incorrectly, the GC may end up collecting objects
/// that are still reachable.
pub unsafe trait Trace {
    unsafe fn trace(&self, trc: *mut JSTracer);
}

unsafe impl Trace for Heap<*mut JSFunction> {
    unsafe fn trace(&self, trc: *mut JSTracer) {
        CallFunctionTracer(trc, self as *const _ as *mut Self, c_str!("function"));
    }
}

unsafe impl Trace for Heap<*mut JSObject> {
    unsafe fn trace(&self, trc: *mut JSTracer) {
        CallObjectTracer(trc, self as *const _ as *mut Self, c_str!("object"));
    }
}

unsafe impl Trace for Heap<*mut JSScript> {
    unsafe fn trace(&self, trc: *mut JSTracer) {
        CallScriptTracer(trc, self as *const _ as *mut Self, c_str!("script"));
    }
}

unsafe impl Trace for Heap<*mut JSString> {
    unsafe fn trace(&self, trc: *mut JSTracer) {
        CallStringTracer(trc, self as *const _ as *mut Self, c_str!("string"));
    }
}

unsafe impl Trace for Heap<Value> {
    unsafe fn trace(&self, trc: *mut JSTracer) {
        CallValueTracer(trc, self as *const _ as *mut Self, c_str!("value"));
    }
}

unsafe impl Trace for Heap<jsid> {
    unsafe fn trace(&self, trc: *mut JSTracer) {
        CallIdTracer(trc, self as *const _ as *mut Self, c_str!("id"));
    }
}

impl<T> Rooted<T> {
    pub fn new_unrooted() -> Rooted<T>
        where T: GCMethods,
    {
        Rooted {
            _base: RootedBase { _phantom0: PhantomData },
            stack: ptr::null_mut(),
            prev: ptr::null_mut(),
            ptr: unsafe { T::initial() },
        }
    }

    pub unsafe fn add_to_root_stack(&mut self, cx: *mut JSContext) where T: RootKind {
        let ctxfriend = cx as *mut ContextFriendFields;
        let zone = (*ctxfriend).zone_;
        let roots: *mut _ = if !zone.is_null() {
            &mut (*zone).stackRoots_
        } else {
            let rt = (*ctxfriend).runtime_;
            let rt = rt as *mut PerThreadDataFriendFields_RuntimeDummy;
            let main_thread = &mut (*rt).mainThread as *mut _;
            let main_thread = main_thread as *mut PerThreadDataFriendFields;
            &mut (*main_thread).roots.stackRoots_
        };

        let kind = T::rootKind() as usize;
        let stack = &mut (*roots)[kind] as *mut _ as *mut _;

        self.stack = stack;
        self.prev = *stack;

        *stack = self as *mut _ as usize as _;
    }

    pub unsafe fn remove_from_root_stack(&mut self) {
        assert!(*self.stack == self as *mut _ as usize as _);
        *self.stack = self.prev;
    }
}

/// Rust API for keeping a Rooted value in the context's root stack.
/// Example usage: `rooted!(in(cx) let x = UndefinedValue());`.
/// `RootedGuard::new` also works, but the macro is preferred.
pub struct RootedGuard<'a, T: 'a + RootKind + GCMethods> {
    root: &'a mut Rooted<T>
}

impl<'a, T: 'a + RootKind + GCMethods> RootedGuard<'a, T> {
    pub fn new(cx: *mut JSContext, root: &'a mut Rooted<T>, initial: T) -> Self {
        root.ptr = initial;
        unsafe {
            root.add_to_root_stack(cx);
        }
        RootedGuard {
            root: root
        }
    }

    // REVIEW: is this equivalent to?
    // pub fn handle(&self) -> Handle<T>
    pub fn handle(&'a self) -> Handle<'a, T> {
        Handle::new(&self.root.ptr)
    }

    pub fn handle_mut(&mut self) -> MutableHandle<T> {
        unsafe {
            MutableHandle::from_marked_location(&mut self.root.ptr)
        }
    }

    pub fn get(&self) -> T where T: Copy {
        self.root.ptr
    }

    pub fn set(&mut self, v: T) {
        self.root.ptr = v;
    }
}

impl<'a, T: 'a + RootKind + GCMethods> Deref for RootedGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.root.ptr
    }
}

impl<'a, T: 'a + RootKind + GCMethods> DerefMut for RootedGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.root.ptr
    }
}

impl<'a, T: 'a + RootKind + GCMethods> Drop for RootedGuard<'a, T> {
    fn drop(&mut self) {
        unsafe {
            self.root.ptr = T::initial();
            self.root.remove_from_root_stack();
        }
    }
}

#[macro_export]
macro_rules! rooted {
    (in($cx:expr) let $name:ident = $init:expr) => {
        let mut __root = $crate::jsapi::Rooted::new_unrooted();
        let $name = $crate::rust::RootedGuard::new($cx, &mut __root, $init);
    };
    (in($cx:expr) let mut $name:ident = $init:expr) => {
        let mut __root = $crate::jsapi::Rooted::new_unrooted();
        let mut $name = $crate::rust::RootedGuard::new($cx, &mut __root, $init);
    }
}

/// Similarly to `Trace` trait, it's used to specify tracing of various types
/// that are used in conjunction with `CustomAutoRooter`.
pub unsafe trait CustomTrace {
    fn trace(&self, trc: *mut JSTracer);
}

unsafe impl CustomTrace for *mut JSObject {
    fn trace(&self, trc: *mut JSTracer) {
        let this = self as *const *mut _ as *mut *mut _;
        unsafe { CallObjectRootTracer(trc, this, c_str!("object")); }
    }
}

unsafe impl CustomTrace for Value {
    fn trace(&self, trc: *mut JSTracer) {
        let this = self as *const _ as *mut _;
        unsafe { CallValueRootTracer(trc, this, c_str!("any")); }
    }
}

unsafe impl<T: CustomTrace> CustomTrace for Option<T> {
    fn trace(&self, trc: *mut JSTracer) {
        if let Some(ref some) = *self {
            some.trace(trc);
        }
    }
}

unsafe impl<T: CustomTrace> CustomTrace for Vec<T> {
    fn trace(&self, trc: *mut JSTracer) {
        for elem in self {
            elem.trace(trc);
        }
    }
}

impl AutoGCRooter {
    pub fn new_unrooted(tag: AutoGCRooterTag) -> AutoGCRooter {
        AutoGCRooter {
            down: ptr::null_mut(),
            tag_: tag as isize,
            stackTop: ptr::null_mut(),
        }
    }

    unsafe fn add_to_root_stack(&mut self, cx: *mut JSContext) {
        let autoGCRooters: &mut _ = {
            let ctxfriend = cx as *mut ContextFriendFields;
            &mut (*ctxfriend).roots.autoGCRooters_
        };
        self.stackTop = autoGCRooters;
        self.down = *autoGCRooters;

        assert!(*self.stackTop != self);
        *self.stackTop = self;
    }

    unsafe fn remove_from_root_stack(&mut self) {
        assert!(*self.stackTop == self);
        *self.stackTop = self.down;
    }
}

// This structure reimplements a C++ class that uses virtual dispatch, so
// use C layout to guarantee that vftable in CustomAutoRooter is in right place.
#[repr(C)]
pub struct CustomAutoRooter<T> {
    _base: jsapi::CustomAutoRooter,
    data: T,
}

impl<T> CustomAutoRooter<T> {
    unsafe fn add_to_root_stack(&mut self, cx: *mut JSContext) {
        self._base._base.add_to_root_stack(cx);
    }

    unsafe fn remove_from_root_stack(&mut self) {
        self._base._base.remove_from_root_stack();
    }
}

/// `CustomAutoRooter` uses dynamic dispatch on the C++ side for custom tracing,
/// so provide trace logic via vftable when creating an object on Rust side.
unsafe trait CustomAutoTraceable: Sized {
    const vftable: CustomAutoRooterVFTable = CustomAutoRooterVFTable {
        trace: Self::trace,
    };

    unsafe extern "C" fn trace(this: *mut ::std::os::raw::c_void, trc: *mut JSTracer) {
        let this = this as *const Self;
        let this = this.as_ref().unwrap();
        Self::do_trace(this, trc);
    }

    /// Used by `CustomAutoTraceable` implementer to trace its contents.
    /// Corresponds to virtual `trace` call in a `CustomAutoRooter` subclass (C++).
    fn do_trace(&self, trc: *mut JSTracer);
}

unsafe impl<T: CustomTrace> CustomAutoTraceable for CustomAutoRooter<T> {
    fn do_trace(&self, trc: *mut JSTracer) {
        self.data.trace(trc);
    }
}

impl<T: CustomTrace> CustomAutoRooter<T> {
    pub fn new(data: T) -> Self {
        CustomAutoRooter {
            _base: jsapi::CustomAutoRooter {
                _vftable: &<Self as CustomAutoTraceable>::vftable,
                _base: AutoGCRooter::new_unrooted(AutoGCRooterTag::CUSTOM),
            },
            data,
        }
    }

    pub fn root<'a>(&'a mut self, cx: *mut JSContext) -> CustomAutoRooterGuard<'a, T> {
        CustomAutoRooterGuard::new(cx, self)
    }
}

/// An RAII guard used to root underlying data in `CustomAutoRooter` until the
/// guard is dropped (falls out of scope).
/// The underlying data can be accessed through this guard via its Deref and
/// DerefMut implementations.
/// This structure is created by `root` method on `CustomAutoRooter` or
/// by the `auto_root!` macro.
pub struct CustomAutoRooterGuard<'a, T: 'a + CustomTrace> {
    rooter: &'a mut CustomAutoRooter<T>
}

impl<'a, T: 'a + CustomTrace> CustomAutoRooterGuard<'a, T> {
    pub fn new(cx: *mut JSContext, rooter: &'a mut CustomAutoRooter<T>) -> Self {
        unsafe {
            rooter.add_to_root_stack(cx);
        }
        CustomAutoRooterGuard {
            rooter
        }
    }

    // REVIEW: as in line 429
    pub fn handle(&'a self) -> Handle<'a, T> where T: RootKind {
        Handle::new(&self.rooter.data)
    }

    pub fn handle_mut(&mut self) -> MutableHandle<T> where T: RootKind {
        unsafe {
            MutableHandle::from_marked_location(&mut self.rooter.data)
        }
    }
}

impl<'a, T: 'a + CustomTrace> Deref for CustomAutoRooterGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.rooter.data
    }
}

impl<'a, T: 'a + CustomTrace> DerefMut for CustomAutoRooterGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.rooter.data
    }
}

impl<'a, T: 'a + CustomTrace> Drop for CustomAutoRooterGuard<'a, T> {
    fn drop(&mut self) {
        unsafe {
            self.rooter.remove_from_root_stack();
        }
    }
}

pub type SequenceRooter<T> = CustomAutoRooter<Vec<T>>;
pub type SequenceRooterGuard<'a, T> = CustomAutoRooterGuard<'a, Vec<T>>;

#[macro_export]
macro_rules! auto_root {
    (in($cx:expr) let $name:ident = $init:expr) => {
        let mut __root = $crate::rust::CustomAutoRooter::new($init);
        let $name = __root.root($cx);
    };
    (in($cx:expr) let mut $name:ident = $init:expr) => {
        let mut __root = $crate::rust::CustomAutoRooter::new($init);
        let mut $name = __root.root($cx);
    }
}

#[derive(Clone, Copy)]
pub struct Handle<'a, T: 'a> {
    ptr: &'a T,
}

pub struct MutableHandle<'a, T: 'a> {
    ptr: &'a mut T
}

pub type MutableHandleValue<'a> = MutableHandle<'a, Value>;
pub type MutableHandleObject<'a> = MutableHandle<'a, *mut JSObject>;

pub type HandleValue<'a> = Handle<'a, Value>;
pub type HandleObject<'a> = Handle<'a, *mut JSObject>;
pub type HandleId<'a> = Handle<'a, jsid>;

impl<T> RawHandle<T> {
    pub fn get(&self) -> T
        where T: Copy
    {
        unsafe { *self.ptr }
    }

    pub unsafe fn from_marked_location(ptr: *const T) -> Self {
        RawHandle {
            _base: HandleBase { _phantom0: PhantomData },
            ptr: ptr,
        }
    }
}

impl<'a, T> Handle<'a, T> {
    pub fn get(&self) -> T
        where T: Copy
    {
        *self.ptr
    }

    pub fn new(ptr: &'a T) -> Self {
        Handle { ptr: ptr }
    }

    pub unsafe fn from_marked_location(ptr: *const T) -> Self {
        Handle::new(&*ptr)
    }

    unsafe fn from_raw(handle: RawHandle<T>) -> Self {
        Handle::from_marked_location(handle.ptr)
    }
}

impl<'a, T> From<Handle<'a, T>> for RawHandle<T> {
    fn from(handle: Handle<'a, T>) -> Self {
        unsafe { RawHandle::from_marked_location(handle.ptr) }
    }
}

impl<'a, T> Deref for Handle<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.ptr
    }
}

impl<T> Deref for RawHandle<T> {
    type Target = T;

    fn deref<'a>(&'a self) -> &'a T {
        unsafe { &*self.ptr }
    }
}

impl<T> RawMutableHandle<T> {
    pub unsafe fn from_marked_location(ptr: *mut T) -> Self {
        Self {
            _base: MutableHandleBase { _phantom0: PhantomData },
            ptr: ptr,
        }
    }

    pub fn handle(&self) -> RawHandle<T> {
        unsafe {
            RawHandle::from_marked_location(self.ptr as *const T)
        }
    }

    pub fn get(&self) -> T
        where T: Copy
    {
        unsafe { *self.ptr }
    }

    pub fn set(&self, v: T)
        where T: Copy
    {
        unsafe { *self.ptr = v }
    }
}

impl<'a, T> MutableHandle<'a, T> {
    pub unsafe fn from_marked_location(ptr: *mut T) -> Self {
        MutableHandle::new(&mut *ptr)
    }

    unsafe fn from_raw(handle: RawMutableHandle<T>) -> Self {
        MutableHandle::from_marked_location(handle.ptr)
    }

    pub fn handle(&self) -> Handle<T> {
        Handle::new(self.ptr)
    }

    pub fn new(ptr: &'a mut T) -> Self {
        Self { ptr: ptr }
    }

    pub fn get(&self) -> T
        where T: Copy
    {
        *self.ptr
    }

    pub fn set(&mut self, v: T)
        where T: Copy
    {
        *self.ptr = v
    }

    fn raw(&mut self) -> RawMutableHandle<T> {
        unsafe {
            RawMutableHandle::from_marked_location(self.ptr)
        }
    }
}

impl<T> Deref for RawMutableHandle<T> {
    type Target = T;

    fn deref<'a>(&'a self) -> &'a T {
        unsafe { &*self.ptr }
    }
}

impl<T> DerefMut for RawMutableHandle<T> {
    fn deref_mut<'a>(&'a mut self) -> &'a mut T {
        unsafe { &mut *self.ptr }
    }
}

impl<'a, T> Deref for MutableHandle<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.ptr
    }
}

impl<'a, T> DerefMut for MutableHandle<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.ptr
    }
}

impl<'a, T> From<MutableHandle<'a, T>> for RawMutableHandle<T> {
    fn from(handle: MutableHandle<'a, T>) -> Self {
        unsafe { RawMutableHandle::from_marked_location(handle.ptr) }
    }
}

impl RawHandleValue {
    pub fn null() -> &'static Self {
        unsafe {
            &NullHandleValue
        }
    }

    pub fn undefined() -> &'static Self {
        unsafe {
            &UndefinedHandleValue
        }
    }
}

impl HandleValue<'static> {
    pub fn null() -> Self {
        Self::new(RawHandleValue::null())
    }

    pub fn undefined() -> Self {
        Self::new(RawHandleValue::undefined())
    }
}

impl HandleValueArray {
    pub fn new() -> HandleValueArray {
        HandleValueArray {
            length_: 0,
            elements_: ptr::null(),
        }
    }

    pub unsafe fn from_rooted_slice(values: &[Value]) -> HandleValueArray {
        HandleValueArray {
            length_: values.len(),
            elements_: values.as_ptr()
        }
    }
}

const ConstNullValue: *mut JSObject = 0 as *mut JSObject;

impl<'a> HandleObject<'a> {
    pub fn null() -> Self {
        unsafe {
            HandleObject::from_marked_location(&ConstNullValue)
        }
    }
}

impl Default for jsid {
    fn default() -> jsid {
        unsafe {
            JSID_VOID
        }
    }
}

impl Default for Value {
    fn default() -> Value { UndefinedValue() }
}

impl Default for CompartmentOptions {
    fn default() -> Self { unsafe { ::std::mem::zeroed() } }
}

const ChunkShift: usize = 20;
const ChunkSize: usize = 1 << ChunkShift;

#[cfg(target_pointer_width = "32")]
const ChunkLocationOffset: usize = ChunkSize - 2 * 4 - 8;

pub trait GCMethods {
    unsafe fn initial() -> Self;
    unsafe fn post_barrier(v: *mut Self, prev: Self, next: Self);
}

impl GCMethods for jsid {
    unsafe fn initial() -> jsid { JSID_VOID }
    unsafe fn post_barrier(_: *mut jsid, _: jsid, _: jsid) {}
}

impl GCMethods for *mut JSObject {
    unsafe fn initial() -> *mut JSObject { ptr::null_mut() }
    unsafe fn post_barrier(v: *mut *mut JSObject,
                           prev: *mut JSObject, next: *mut JSObject) {
        HeapObjectPostBarrier(v, prev, next);
    }
}

impl GCMethods for *mut JSString {
    unsafe fn initial() -> *mut JSString { ptr::null_mut() }
    unsafe fn post_barrier(_: *mut *mut JSString, _: *mut JSString, _: *mut JSString) {}
}

impl GCMethods for *mut JSScript {
    unsafe fn initial() -> *mut JSScript { ptr::null_mut() }
    unsafe fn post_barrier(_: *mut *mut JSScript, _: *mut JSScript, _: *mut JSScript) { }
}

impl GCMethods for *mut JSFunction {
    unsafe fn initial() -> *mut JSFunction { ptr::null_mut() }
    unsafe fn post_barrier(v: *mut *mut JSFunction,
                           prev: *mut JSFunction, next: *mut JSFunction) {
        HeapObjectPostBarrier(mem::transmute(v),
                              mem::transmute(prev), mem::transmute(next));
    }
}

impl GCMethods for Value {
    unsafe fn initial() -> Value { UndefinedValue() }
    unsafe fn post_barrier(v: *mut Value, prev: Value, next: Value) {
        HeapValuePostBarrier(v, &prev, &next);
    }
}

impl<T: GCMethods + Copy> Heap<T> {
    /// This creates a `Box`-wrapped Heap value. Setting a value inside Heap
    /// object triggers a barrier, referring to the Heap object location,
    /// hence why it is not safe to construct a temporary Heap value, assign
    /// a non-null value and move it (e.g. typical object construction).
    ///
    /// Using boxed Heap value guarantees that the underlying Heap value will
    /// not be moved when constructed.
    pub fn boxed(v: T) -> Box<Heap<T>>
        where Heap<T>: Default
    {
        let boxed = Box::new(Heap::default());
        boxed.set(v);
        boxed
    }

    pub fn set(&self, v: T) {
        unsafe {
            let ptr = self.ptr.get();
            let prev = *ptr;
            *ptr = v;
            T::post_barrier(ptr, prev, v);
        }
    }

    pub fn get(&self) -> T {
        unsafe { *self.ptr.get() }
    }

    pub fn get_unsafe(&self) -> *mut T {
        self.ptr.get()
    }

    pub fn handle(&self) -> Handle<T> {
        unsafe {
            Handle::from_marked_location(self.ptr.get() as *const T)
        }
    }

    pub fn handle_mut(&self) -> MutableHandle<T> {
        unsafe {
            MutableHandle::from_marked_location(self.ptr.get())
        }
    }
}

impl<T> Default for Heap<*mut T>
    where *mut T: GCMethods + Copy
{
    fn default() -> Heap<*mut T> {
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

impl<T: GCMethods + Copy> Drop for Heap<T> {
    fn drop(&mut self) {
        unsafe {
            let ptr = self.ptr.get();
            T::post_barrier(ptr, *ptr, T::initial());
        }
    }
}

impl<T: GCMethods + Copy + PartialEq> PartialEq for Heap<T> {
    fn eq(&self, other: &Self) -> bool {
        self.get() == other.get()
    }
}

// ___________________________________________________________________________
// Implementations for various things in jsapi.rs

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
    #[inline]
    pub fn get(&self, i: u32) -> HandleValue {
        unsafe {
            if i < self._base.argc_ {
                HandleValue::from_marked_location(
                    self._base.argv_.offset(i as isize)
                )
            } else {
                HandleValue::from_raw(UndefinedHandleValue)
            }
        }
    }

    #[inline]
    pub fn index(&self, i: u32) -> HandleValue {
        assert!(i < self._base.argc_);
        unsafe {
            HandleValue::from_marked_location(self._base.argv_.offset(i as isize))
        }
    }

    #[inline]
    pub fn index_mut(&self, i: u32) -> MutableHandleValue {
        assert!(i < self._base.argc_);
        unsafe {
            MutableHandleValue::from_marked_location(self._base.argv_.offset(i as isize))
        }
    }

    #[inline]
    pub fn rval(&self) -> MutableHandleValue {
        unsafe {
            MutableHandleValue::from_marked_location(self._base.argv_.offset(-2))
        }
    }
}

// XXX need to hack up bindgen to convert this better so we don't have
//     to duplicate so much code here
impl CallArgs {
    #[inline]
    pub unsafe fn from_vp(vp: *mut Value, argc: u32) -> CallArgs {
        CreateCallArgsFromVp(argc, vp)
    }

    #[inline]
    pub fn index(&self, i: u32) -> HandleValue {
        assert!(i < self._base.argc_);
        unsafe {
            HandleValue::from_marked_location(self._base.argv_.offset(i as isize))
        }
    }

    #[inline]
    pub fn index_mut(&self, i: u32) -> MutableHandleValue {
        assert!(i < self._base.argc_);
        unsafe {
            MutableHandleValue::from_marked_location(self._base.argv_.offset(i as isize))
        }
    }

    #[inline]
    pub fn get(&self, i: u32) -> HandleValue {
        unsafe {
            if i < self._base.argc_ {
                HandleValue::from_marked_location(self._base.argv_.offset(i as isize))
            } else {
                HandleValue::from_raw(UndefinedHandleValue)
            }
        }
    }

    #[inline]
    pub fn rval(&self) -> MutableHandleValue {
        unsafe {
            MutableHandleValue::from_marked_location(self._base.argv_.offset(-2))
        }
    }

    #[inline]
    pub fn thisv(&self) -> HandleValue {
        unsafe {
            HandleValue::from_marked_location(self._base.argv_.offset(-1))
        }
    }

    #[inline]
    pub fn calleev(&self) -> HandleValue {
        unsafe {
            HandleValue::from_marked_location(self._base.argv_.offset(-2))
        }
    }

    #[inline]
    pub fn callee(&self) -> *mut JSObject {
        self.calleev().to_object()
    }

    #[inline]
    pub fn new_target(&self) -> MutableHandleValue {
        assert!(self._base.constructing_);
        unsafe {
            MutableHandleValue::from_marked_location(self._base.argv_.offset(self._base.argc_ as isize))
        }
    }
}

impl JSJitGetterCallArgs {
    #[inline]
    pub fn rval(&self) -> MutableHandleValue {
        unsafe {
            MutableHandleValue::from_raw(self._base)
        }
    }
}

impl JSJitSetterCallArgs {
    #[inline]
    pub fn get(&self, i: u32) -> HandleValue {
        assert!(i == 0);
        unsafe {
            HandleValue::from_raw(self._base.handle())
        }
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
            AppendToAutoObjectVector(self.ptr, obj)
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
    pub fn new(cx: *mut JSContext, file: *const ::libc::c_char, line: c_uint) -> CompileOptionsWrapper {
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
pub unsafe fn ToBoolean(v: HandleValue) -> bool {
    let val = *v.ptr;

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

    ToBooleanSlow(v.into())
}

#[inline]
pub unsafe fn ToNumber(cx: *mut JSContext, v: HandleValue) -> Result<f64, ()> {
    let val = *v.ptr;
    if val.is_number() {
        return Ok(val.to_number());
    }

    let mut out = Default::default();
    if ToNumberSlow(cx, val, &mut out) {
        Ok(out)
    } else {
        Err(())
    }
}

#[inline]
unsafe fn convert_from_int32<T: Default + Copy>(
    cx: *mut JSContext,
    v: HandleValue,
    conv_fn: unsafe extern "C" fn(*mut JSContext, RawHandleValue, *mut T) -> bool)
        -> Result<T, ()> {

    let val = *v.ptr;
    if val.is_int32() {
        let intval: i64 = val.to_int32() as i64;
        // TODO: do something better here that works on big endian
        let intval = *(&intval as *const i64 as *const T);
        return Ok(intval);
    }

    let mut out = Default::default();
    if conv_fn(cx, v.into(), &mut out) {
        Ok(out)
    } else {
        Err(())
    }
}

#[inline]
pub unsafe fn ToInt32(cx: *mut JSContext, v: HandleValue) -> Result<i32, ()> {
    convert_from_int32::<i32>(cx, v, ToInt32Slow)
}

#[inline]
pub unsafe fn ToUint32(cx: *mut JSContext, v: HandleValue) -> Result<u32, ()> {
    convert_from_int32::<u32>(cx, v, ToUint32Slow)
}

#[inline]
pub unsafe fn ToUint16(cx: *mut JSContext, v: HandleValue) -> Result<u16, ()> {
    convert_from_int32::<u16>(cx, v, ToUint16Slow)
}

#[inline]
pub unsafe fn ToInt64(cx: *mut JSContext, v: HandleValue) -> Result<i64, ()> {
    convert_from_int32::<i64>(cx, v, ToInt64Slow)
}

#[inline]
pub unsafe fn ToUint64(cx: *mut JSContext, v: HandleValue) -> Result<u64, ()> {
    convert_from_int32::<u64>(cx, v, ToUint64Slow)
}

#[inline]
pub unsafe fn ToString(cx: *mut JSContext, v: HandleValue) -> *mut JSString {
    let val = *v.ptr;
    if val.is_string() {
        return val.to_string();
    }

    ToStringSlow(cx, v.into())
}

pub unsafe extern fn report_warning(_cx: *mut JSContext, _: *const c_char, report: *mut JSErrorReport) {
    fn latin1_to_string(bytes: &[u8]) -> String {
        bytes.iter().map(|c| char::from_u32(*c as u32).unwrap()).collect()
    }

    let fnptr = (*report).filename;
    let fname = if !fnptr.is_null() {
        let c_str = ffi::CStr::from_ptr(fnptr);
        latin1_to_string(c_str.to_bytes())
    } else {
        "none".to_string()
    };

    let lineno = (*report).lineno;
    let column = (*report).column;

    let msg_ptr = (*report).ucmessage;
    let msg_len = (0usize..).find(|&i| *msg_ptr.offset(i as isize) == 0).unwrap();
    let msg_slice = slice::from_raw_parts(msg_ptr, msg_len);
    let msg = String::from_utf16_lossy(msg_slice);

    warn!("Warning at {}:{}:{}: {}\n", fname, lineno, column, msg);
}

impl JSNativeWrapper {
    fn is_zeroed(&self) -> bool {
        let JSNativeWrapper { op, info } = *self;
        op.is_none() && info.is_null()
    }
}

pub struct IdVector(*mut AutoIdVector);

impl IdVector {
    pub unsafe fn new(cx: *mut JSContext) -> IdVector {
        let vector = CreateAutoIdVector(cx);
        assert!(!vector.is_null());
        IdVector(vector)
    }

    pub fn get(&self) -> *mut AutoIdVector {
        self.0
    }
}

impl Drop for IdVector {
    fn drop(&mut self) {
        unsafe {
            DestroyAutoIdVector(self.0)
        }
    }
}

impl Deref for IdVector {
    type Target = [jsid];

    fn deref(&self) -> &[jsid] {
        unsafe {
            let mut length = 0;
            let pointer = SliceAutoIdVector(self.0 as *const _, &mut length);
            slice::from_raw_parts(pointer, length)
        }
    }
}

/// Defines methods on `obj`. The last entry of `methods` must contain zeroed
/// memory.
///
/// # Failures
///
/// Returns `Err` on JSAPI failure.
///
/// # Panics
///
/// Panics if the last entry of `methods` does not contain zeroed memory.
///
/// # Safety
///
/// - `cx` must be valid.
/// - This function calls into unaudited C++ code.
pub unsafe fn define_methods(cx: *mut JSContext, obj: HandleObject,
                             methods: &'static [JSFunctionSpec])
                             -> Result<(), ()> {
    assert!({
        match methods.last() {
            Some(&JSFunctionSpec { name, call, nargs, flags, selfHostedName }) => {
                name.is_null() && call.is_zeroed() && nargs == 0 && flags == 0 &&
                selfHostedName.is_null()
            },
            None => false,
        }
    });

    JS_DefineFunctions(cx, obj.into(), methods.as_ptr()).to_result()
}

/// Defines attributes on `obj`. The last entry of `properties` must contain
/// zeroed memory.
///
/// # Failures
///
/// Returns `Err` on JSAPI failure.
///
/// # Panics
///
/// Panics if the last entry of `properties` does not contain zeroed memory.
///
/// # Safety
///
/// - `cx` must be valid.
/// - This function calls into unaudited C++ code.
pub unsafe fn define_properties(cx: *mut JSContext, obj: HandleObject,
                                properties: &'static [JSPropertySpec])
                                -> Result<(), ()> {
    assert!({
        match properties.last() {
            Some(&JSPropertySpec { name, flags, getter, setter }) => {
                name.is_null() && flags == 0 && getter.is_zeroed() && setter.is_zeroed()
            },
            None => false,
        }
    });

    JS_DefineProperties(cx, obj.into(), properties.as_ptr()).to_result()
}

static SIMPLE_GLOBAL_CLASS_OPS: JSClassOps = JSClassOps {
    addProperty: None,
    delProperty: None,
    getProperty: None,
    setProperty: None,
    enumerate: Some(JS_EnumerateStandardClasses),
    resolve: Some(JS_ResolveStandardClass),
    mayResolve: Some(JS_MayResolveStandardClass),
    finalize: None,
    call: None,
    hasInstance: None,
    construct: None,
    trace: Some(JS_GlobalObjectTraceHook),
};

/// This is a simple `JSClass` for global objects, primarily intended for tests.
pub static SIMPLE_GLOBAL_CLASS: JSClass = JSClass {
    name: b"Global\0" as *const u8 as *const _,
    flags: JSCLASS_IS_GLOBAL | ((JSCLASS_GLOBAL_SLOT_COUNT & JSCLASS_RESERVED_SLOTS_MASK) << JSCLASS_RESERVED_SLOTS_SHIFT),
    cOps: &SIMPLE_GLOBAL_CLASS_OPS as *const JSClassOps,
    reserved: [0 as *mut _; 3]
};

#[inline]
unsafe fn get_object_group(obj: *mut JSObject) -> *mut ObjectGroup {
    assert!(!obj.is_null());
    let obj = obj as *mut Object;
    (*obj).group
}

#[inline]
pub unsafe fn get_object_class(obj: *mut JSObject) -> *const JSClass {
    (*get_object_group(obj)).clasp as *const _
}

#[inline]
pub unsafe fn get_object_compartment(obj: *mut JSObject) -> *mut JSCompartment {
    (*get_object_group(obj)).compartment
}

#[inline]
pub unsafe fn get_context_compartment(cx: *mut JSContext) -> *mut JSCompartment {
    let cx = cx as *mut ContextFriendFields;
    (*cx).compartment_
}

#[inline]
pub fn is_dom_class(class: &JSClass) -> bool {
    class.flags & JSCLASS_IS_DOMJSCLASS != 0
}

#[inline]
pub unsafe fn is_dom_object(obj: *mut JSObject) -> bool {
    is_dom_class(&*get_object_class(obj))
}

#[inline]
pub unsafe fn is_window(obj: *mut JSObject) -> bool {
    (*get_object_class(obj)).flags & JSCLASS_IS_GLOBAL != 0 && IsWindowSlow(obj)
}

#[inline]
pub unsafe fn try_to_outerize(mut rval: MutableHandleValue) {
    let obj = rval.to_object();
    if is_window(obj) {
        let obj = ToWindowProxyIfWindow(obj);
        assert!(!obj.is_null());
        rval.set(ObjectValue(&mut *obj));
    }
}

#[inline]
pub unsafe fn maybe_wrap_object_value(cx: *mut JSContext, rval: MutableHandleValue) {
    assert!(rval.is_object());
    let obj = rval.to_object();
    if get_object_compartment(obj) != get_context_compartment(cx) {
        assert!(JS_WrapValue(cx, rval.into()));
    } else if is_dom_object(obj) {
        try_to_outerize(rval);
    }
}

#[inline]
pub unsafe fn maybe_wrap_object_or_null_value(
        cx: *mut JSContext,
        rval: MutableHandleValue) {
    assert!(rval.is_object_or_null());
    if !rval.is_null() {
        maybe_wrap_object_value(cx, rval);
    }
}

#[inline]
pub unsafe fn maybe_wrap_value(cx: *mut JSContext, rval: MutableHandleValue) {
    if rval.is_string() {
        assert!(JS_WrapValue(cx, rval.into()));
    } else if rval.is_object() {
        maybe_wrap_object_value(cx, rval);
    }
}

/// Like `JSJitInfo::new_bitfield_1`, but usable in `const` contexts.
#[macro_export]
macro_rules! new_jsjitinfo_bitfield_1 {
    (
        $type_: expr,
        $aliasSet_: expr,
        $returnType_: expr,
        $isInfallible: expr,
        $isMovable: expr,
        $isEliminatable: expr,
        $isAlwaysInSlot: expr,
        $isLazilyCachedInSlot: expr,
        $isTypedMethod: expr,
        $slotIndex: expr,
    ) => {
        0 | (($type_ as u32) << 0u32) |
            (($aliasSet_ as u32) << 4u32) |
            (($returnType_ as u32) << 8u32) |
            (($isInfallible as u32) << 16u32) |
            (($isMovable as u32) << 17u32) |
            (($isEliminatable as u32) << 18u32) |
            (($isAlwaysInSlot as u32) << 19u32) |
            (($isLazilyCachedInSlot as u32) << 20u32) |
            (($isTypedMethod as u32) << 21u32) |
            (($slotIndex as u32) << 22u32)
    }
}

pub struct CapturedJSStack<'a> {
    cx: *mut JSContext,
    stack: RootedGuard<'a, *mut JSObject>,
}

impl<'a> CapturedJSStack<'a> {
    pub unsafe fn new(cx: *mut JSContext,
                      mut guard: RootedGuard<'a, *mut JSObject>,
                      max_frame_count: Option<u32>) -> Option<Self> {
        if !CaptureCurrentStack(cx, guard.handle_mut().raw(), max_frame_count.unwrap_or(0)) {
            None
        }
        else {
            Some(CapturedJSStack {
                cx: cx,
                stack: guard,
            })
        }
    }

    pub fn as_string(&self, indent: Option<usize>) -> Option<String> {
        unsafe {
            let stack_handle = self.stack.handle();
            rooted!(in(self.cx) let mut js_string = ptr::null_mut::<JSString>());
            let mut string_handle = js_string.handle_mut();

            if !IsSavedFrame(stack_handle.get()) {
                return None;
            }

            if !BuildStackString(self.cx, stack_handle.into(), string_handle.raw(), indent.unwrap_or(0)) {
                return None;
            }

            Some(jsstr_to_string(self.cx, string_handle.get()))
        }
    }
}

#[macro_export]
macro_rules! capture_stack {
    (in($cx:expr) let $name:ident = with max depth($max_frame_count:expr)) => {
        rooted!(in($cx) let mut __obj = ::std::ptr::null_mut());
        let $name = $crate::rust::CapturedJSStack::new($cx, __obj, Some($max_frame_count));
    };
    (in($cx:expr) let $name:ident ) => {
        rooted!(in($cx) let mut __obj = ::std::ptr::null_mut());
        let $name = $crate::rust::CapturedJSStack::new($cx, __obj, None);
    }
}

macro_rules! wrap {
    // The invocation of @inner has the following form:
    // @inner (input args) <> (accumulator) <> unparsed tokens
    // when `unparsed tokens == \eps`, accumulator contains the final result

    (@inner $saved:tt <> ($($acc:expr,)*) <> $arg:ident: Handle<$gentype:ty>, $($rest:tt)*) => {
        wrap!(@inner $saved <> ($($acc,)* $arg.into(),) <> $($rest)*);
    };
    (@inner $saved:tt <> ($($acc:expr,)*) <> $arg:ident: MutableHandle<$gentype:ty>, $($rest:tt)*) => {
        wrap!(@inner $saved <> ($($acc,)* $arg.into(),) <> $($rest)*);
    };
    (@inner $saved:tt <> ($($acc:expr,)*) <> $arg:ident: Handle, $($rest:tt)*) => {
        wrap!(@inner $saved <> ($($acc,)* $arg.into(),) <> $($rest)*);
    };
    (@inner $saved:tt <> ($($acc:expr,)*) <> $arg:ident: HandleValue, $($rest:tt)*) => {
        wrap!(@inner $saved <> ($($acc,)* $arg.into(),) <> $($rest)*);
    };
    (@inner $saved:tt <> ($($acc:expr,)*) <> $arg:ident: HandleObject, $($rest:tt)*) => {
        wrap!(@inner $saved <> ($($acc,)* $arg.into(),) <> $($rest)*);
    };
    (@inner $saved:tt <> ($($acc:expr,)*) <> $arg:ident: HandleId, $($rest:tt)*) => {
        wrap!(@inner $saved <> ($($acc,)* $arg.into(),) <> $($rest)*);
    };
    (@inner $saved:tt <> ($($acc:expr,)*) <> $arg:ident: MutableHandle, $($rest:tt)*) => {
        wrap!(@inner $saved <> ($($acc,)* $arg.into(),) <> $($rest)*);
    };
    (@inner $saved:tt <> ($($acc:expr,)*) <> $arg:ident: MutableHandleObject, $($rest:tt)*) => {
        wrap!(@inner $saved <> ($($acc,)* $arg.into(),) <> $($rest)*);
    };
    (@inner $saved:tt <> ($($acc:expr,)*) <> $arg:ident: MutableHandleValue, $($rest:tt)*) => {
        wrap!(@inner $saved <> ($($acc,)* $arg.into(),) <> $($rest)*);
    };
    (@inner $saved:tt <> ($($acc:expr,)*) <> $arg:ident: $type:ty, $($rest:tt)*) => {
        wrap!(@inner $saved <> ($($acc,)* $arg,) <> $($rest)*);
    };
    (@inner ($func_name:ident ($($args:tt)*) -> $outtype:ty) <> ($($argexprs:expr,)*) <> ) => {
        pub unsafe fn $func_name($($args)*) -> $outtype {
            jsapi::$func_name($($argexprs),*)
        }
    };
    (pub fn $func_name:ident($($args:tt)*) -> $outtype:ty) => {
        wrap!(@inner ($func_name ($($args)*) -> $outtype) <> () <> $($args)* ,);
    };
    (pub fn $func_name:ident($($args:tt)*)) => {
        wrap!(pub fn $func_name($($args)*) -> ());
    }
}
pub mod wrappers {
    use jsapi;
    use jsapi::{IsArrayAnswer, PropertyDescriptor, ElementAdder, DetachDataDisposition};
    use jsapi::{JSStructuredCloneCallbacks, JSStructuredCloneReader, JSStructuredCloneWriter};
    use jsapi::{JSNative, JSObject, JSContext, JSFunction, JSRuntime, JSString};
    use jsapi::{JSType};
    use jsapi::{HandleString, HandleScript};
    use jsapi::{SavedFrameResult, SavedFrameSelfHosted};
    use jsapi::{MallocSizeOf, ObjectPrivateVisitor, ObjectOpResult, TabSizes};
    use jsapi::AutoIdVector;
    use jsapi::AutoObjectVector;
    use jsapi::CallArgs;
    use jsapi::CompileOptions;
    use jsapi::ESClass;
    use jsapi::HandleFunction;
    use jsapi::HandleSymbol;
    use jsapi::HandleValueArray;
    use jsapi::JSAddonId;
    use jsapi::JSClass;
    use jsapi::JSConstDoubleSpec;
    use jsapi::JSConstIntegerSpec;
    use jsapi::JSErrorReport;
    use jsapi::JSExnType;
    use jsapi::JSFunctionSpec;
    use jsapi::JSFunctionSpecWithHelp;
    use jsapi::jsid;
    use jsapi::JSONWriteCallback;
    use jsapi::JSPropertySpec;
    use jsapi::JSProtoKey;
    use jsapi::JSScript;
    use jsapi::MutableHandleFunction;
    use jsapi::MutableHandleId;
    use jsapi::MutableHandleScript;
    use jsapi::MutableHandleString;
    use jsapi::PromiseState;
    use jsapi::PropertyCopyBehavior;
    use jsapi::ReadOnlyCompileOptions;
    use jsapi::RegExpGuard;
    use jsapi::ScriptEnvironmentPreparer_Closure;
    use jsapi::Shape;
    use jsapi::SourceBufferHolder;
    use jsapi::Symbol;
    use jsapi::SymbolCode;
    use jsapi::TwoByteChars;
    use jsapi::Value;
    use libc::FILE;
    use super::{Handle, HandleId, HandleObject, HandleValue};
    use super::{MutableHandle, MutableHandleObject, MutableHandleValue};
    include!("jsapi_wrappers.in");
}
