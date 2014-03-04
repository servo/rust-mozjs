/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/. */

#[doc = "Rust wrappers around the raw JS apis"];

use std::libc::types::os::arch::c95::{size_t, c_uint};
use std::libc::{c_char, uintptr_t};
use std::cmp;
use std::rc;
use jsapi::*;
use jsval::{JSVal, NullValue};
use default_stacksize;
use default_heapsize;
use JSOPTION_VAROBJFIX;
use JSOPTION_METHODJIT;
use JSOPTION_TYPE_INFERENCE;
use ERR;
use std::ptr;
use std::ptr::null;
use result;
use result_obj;
use std::str::raw::from_c_str;
use std::cast;
use green::task::GreenTask;

// ___________________________________________________________________________
// friendly Rustic API to runtimes

pub type rt = rc::Rc<rt_rsrc>;

pub struct rt_rsrc {
    ptr : *JSRuntime,
}

impl Drop for rt_rsrc {
    fn drop(&mut self) {
        unsafe {
            JS_Finish(self.ptr);
        }
    }
}

pub fn new_runtime(p: *JSRuntime) -> rt {
    return rc::Rc::new(rt_rsrc {
        ptr: p
    })
}

pub trait RtUtils {
    fn cx(&self) -> rc::Rc<Cx>;
}

impl RtUtils for rc::Rc<rt_rsrc> {
    fn cx(&self) -> rc::Rc<Cx> {
        unsafe {
            new_context(JS_NewContext(self.borrow().ptr,
                                      default_stacksize as size_t), self.clone())
        }
    }
}

// FIXME: Is this safe once we have more than one stack segment?
extern fn gc_callback(rt: *JSRuntime, _status: JSGCStatus) {
    use std::rt::local::Local;
    use std::rt::task::Task;
    unsafe {
        let mut task = Local::borrow(None::<Task>);
        let green_task: ~GreenTask = task.get().maybe_take_runtime().unwrap();
        {
            let c = green_task.coroutine.get_ref();
            let start = c.current_stack_segment.start() as uintptr_t;
            let end = c.current_stack_segment.end() as uintptr_t;
            JS_SetNativeStackBounds(rt, cmp::min(start, end), cmp::max(start, end));
        }
        task.get().put_runtime(green_task);
    }
}

pub fn rt() -> rt {
    unsafe {
        let runtime = JS_Init(default_heapsize);
        JS_SetGCCallback(runtime, gc_callback);
        return new_runtime(runtime);
    }
}

// ___________________________________________________________________________
// contexts

pub struct Cx {
    ptr: *JSContext,
    rt: rt,
}

#[unsafe_destructor]
impl Drop for Cx {
    fn drop(&mut self) {
        unsafe {
            JS_DestroyContext(self.ptr);
        }
    }
}

pub fn new_context(ptr: *JSContext, rt: rt) -> rc::Rc<Cx> {
    return rc::Rc::new(Cx {
        ptr: ptr,
        rt: rt,
    })
}

pub trait CxUtils {
    fn rooted_obj(&self, obj: *JSObject) -> jsobj;
    fn new_compartment(&self, globcls: *JSClass) -> Result<rc::Rc<Compartment>,()>;
    fn new_compartment_with_global(&self, global: *JSObject) -> Result<rc::Rc<Compartment>,()>;
}

impl CxUtils for rc::Rc<Cx> {
    fn rooted_obj(&self, obj: *JSObject) -> jsobj {
        let cxptr = self.borrow().ptr;
        let jsobj = rc::Rc::new(jsobj_rsrc {cx: self.clone(), cxptr: cxptr, ptr: obj});
        unsafe {
            JS_AddObjectRoot(cxptr, &jsobj.borrow().ptr);
        }
        jsobj
    }

    fn new_compartment(&self, globcls: *JSClass) -> Result<rc::Rc<Compartment>,()> {
        unsafe {
            let ptr = self.borrow().ptr;
            let globobj = JS_NewGlobalObject(ptr, globcls, null());
            result(JS_InitStandardClasses(ptr, globobj)).and_then(|_ok| {
                Ok(rc::Rc::new(Compartment {
                    cx: self.clone(),
                    global_obj: self.rooted_obj(globobj),
                }))
            })
        }
    }

    fn new_compartment_with_global(&self, global: *JSObject) -> Result<rc::Rc<Compartment>,()> {
        Ok(rc::Rc::new(Compartment {
            cx: self.clone(),
            global_obj: self.rooted_obj(global),
        }))
    }
}

impl Cx {
    pub fn set_default_options_and_version(&self) {
        self.set_options(JSOPTION_VAROBJFIX | JSOPTION_METHODJIT |
                         JSOPTION_TYPE_INFERENCE);
        self.set_version(JSVERSION_LATEST);
    }

    pub fn set_options(&self, v: c_uint) {
        unsafe {
            JS_SetOptions(self.ptr, v);
        }
    }

    pub fn set_version(&self, v: i32) {
        unsafe {
            JS_SetVersion(self.ptr, v);
        }
    }

    pub fn set_logging_error_reporter(&self) {
        unsafe {
            JS_SetErrorReporter(self.ptr, reportError);
        }
    }

    pub fn set_error_reporter(&self, reportfn: extern "C" fn(*JSContext, *c_char, *JSErrorReport)) {
        unsafe {
            JS_SetErrorReporter(self.ptr, reportfn);
        }
    }

    pub fn evaluate_script(&self, glob: jsobj, script: ~str, filename: ~str, line_num: uint)
                    -> Result<(),()> {
        let script_utf16 = script.to_utf16();
        filename.to_c_str().with_ref(|filename_cstr| {
            let rval: JSVal = NullValue();
            debug!("Evaluating script from {:s} with content {}", filename, script);
            unsafe {
                if ERR == JS_EvaluateUCScript(self.ptr, glob.borrow().ptr,
                                              script_utf16.as_ptr(), script_utf16.len() as c_uint,
                                              filename_cstr, line_num as c_uint,
                                              &rval) {
                    debug!("...err!");
                    Err(())
                } else {
                    // we could return the script result but then we'd have
                    // to root it and so forth and, really, who cares?
                    debug!("...ok!");
                    Ok(())
                }
            }
        })
    }

    pub unsafe fn get_cx_private(&self) -> *() {
        cast::transmute(JS_GetContextPrivate(self.ptr))
    }

    pub unsafe fn set_cx_private(&self, data: *()) {
        JS_SetContextPrivate(self.ptr, cast::transmute(data));
    }

    pub unsafe fn get_obj_private(&self, obj: *JSObject) -> *() {
        cast::transmute(JS_GetPrivate(obj))
    }

    pub unsafe fn set_obj_private(&self, obj: *JSObject, data: *()) {
        JS_SetPrivate(obj, cast::transmute(data));
    }
}

pub extern fn reportError(_cx: *JSContext, msg: *c_char, report: *JSErrorReport) {
    unsafe {
        let fnptr = (*report).filename;
        let fname = if fnptr.is_not_null() {from_c_str(fnptr)} else {~"none"};
        let lineno = (*report).lineno;
        let msg = from_c_str(msg);
        error!("Error at {:s}:{}: {:s}\n", fname, lineno, msg);
    }
}

// ___________________________________________________________________________
// compartment

pub struct Compartment {
    cx: rc::Rc<Cx>,
    global_obj: jsobj,
}

impl Compartment {
    pub fn define_functions(&self, specvec: &'static [JSFunctionSpec]) -> Result<(),()> {
        unsafe {
            result(JS_DefineFunctions(self.cx.borrow().ptr,
                                      self.global_obj.borrow().ptr,
                                      specvec.as_ptr()))
        }
    }
    pub fn define_properties(&self, specvec: &'static [JSPropertySpec]) -> Result<(),()> {
        unsafe {
            result(JS_DefineProperties(self.cx.borrow().ptr,
                                       self.global_obj.borrow().ptr,
                                       specvec.as_ptr()))
        }
    }
    pub fn define_property(&self,
                           name: &'static str,
                           value: JSVal,
                           getter: JSPropertyOp, setter: JSStrictPropertyOp,
                           attrs: c_uint)
        -> Result<(),()> {
        unsafe {
            name.to_c_str().with_ref(|name| {
                result(JS_DefineProperty(self.cx.borrow().ptr,
                                         self.global_obj.borrow().ptr,
                                         name,
                                         value,
                                         Some(getter),
                                         Some(setter),
                                         attrs))
            })
        }
    }
    pub fn new_object(&self, classptr: *JSClass, proto: *JSObject, parent: *JSObject)
               -> Result<jsobj, ()> {
        unsafe {
            let obj = self.cx.rooted_obj(JS_NewObject(self.cx.borrow().ptr, classptr, proto, parent));
            result_obj(obj)
        }
    }
}

// ___________________________________________________________________________
// objects

pub type jsobj = rc::Rc<jsobj_rsrc>;

pub struct jsobj_rsrc {
    cx: rc::Rc<Cx>,
    cxptr: *JSContext,
    ptr: *JSObject,
}

#[unsafe_destructor]
impl Drop for jsobj_rsrc {
    fn drop(&mut self) {
        unsafe {
            JS_RemoveObjectRoot(self.cxptr, &self.ptr);
        }
    }
}

impl jsobj_rsrc {
    pub fn new_object(&self, cx: rc::Rc<Cx>, cxptr: *JSContext, ptr: *JSObject) -> jsobj {
        return rc::Rc::new(jsobj_rsrc {
            cx: cx,
            cxptr: cxptr,
            ptr: ptr
        })
    }
}

// ___________________________________________________________________________
// random utilities

pub trait to_jsstr {
    fn to_jsstr(self, cx: rc::Rc<Cx>) -> *JSString;
}

impl to_jsstr for ~str {
    fn to_jsstr(self, cx: rc::Rc<Cx>) -> *JSString {
        unsafe {
            let cbuf = cast::transmute(self.as_ptr());
            JS_NewStringCopyN(cx.borrow().ptr, cbuf, self.len() as size_t)
        }
    }
}

#[cfg(test)]
pub mod test {
    use super::rt;
    use super::{CxUtils, RtUtils};
    use super::super::global;
    use super::super::jsapi::{JS_GC, JS_GetRuntime};

    #[test]
    pub fn dummy() {
        let rt = rt();
        let cx = rt.cx();
        cx.borrow().set_default_options_and_version();
        cx.borrow().set_logging_error_reporter();
        cx.new_compartment(&global::BASIC_GLOBAL).and_then(|comp| {
            unsafe { JS_GC(JS_GetRuntime(cx.borrow().ptr)); }

            comp.borrow().define_functions(global::DEBUG_FNS);

            let s = ~"debug(22);";
            cx.borrow().evaluate_script(comp.borrow().global_obj.clone(), s, ~"test", 1u)
        });
    }

}
