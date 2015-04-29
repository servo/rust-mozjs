/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Rust wrappers around the raw JS apis

use libc::types::os::arch::c95::{size_t, c_uint};
use libc::c_char;
use std::ffi;
use std::rc;
use std::str;
use std::u32;
use jsapi::*;
use jsapi::JSVersion::JSVERSION_LATEST;
use jsval::{JSVal, NullValue};
use default_stacksize;
use default_heapsize;
use JSOPTION_VAROBJFIX;
use JSOPTION_METHODJIT;
use JSOPTION_TYPE_INFERENCE;
use JSOPTION_DONT_REPORT_UNCAUGHT;
use JSOPTION_AUTOJSAPI_OWNS_ERROR_REPORTING;
use ERR;

// ___________________________________________________________________________
// friendly Rustic API to runtimes

/// A wrapper for the `JSRuntime` and `JSContext` structures in SpiderMonkey.
pub struct Runtime {
    pub rt: rt,
    pub cx: rc::Rc<Cx>,
}

impl Runtime {
    /// Creates a new `JSRuntime` and `JSContext`.
    pub fn new() -> Runtime {
        let js_runtime = rt();
        assert!({
            let ptr: *mut JSRuntime = (*js_runtime).ptr;
            !ptr.is_null()
        });

        // Unconstrain the runtime's threshold on nominal heap size, to avoid
        // triggering GC too often if operating continuously near an arbitrary
        // finite threshold. This leaves the maximum-JS_malloc-bytes threshold
        // still in effect to cause periodical, and we hope hygienic,
        // last-ditch GCs from within the GC's allocator.
        unsafe {
            JS_SetGCParameter(js_runtime.ptr, JSGC_MAX_BYTES, u32::MAX);
        }

        let js_context = js_runtime.cx();
        assert!({
            let ptr: *mut JSContext = (*js_context).ptr;
            !ptr.is_null()
        });
        js_context.set_default_options_and_version();
        js_context.set_logging_error_reporter();
        unsafe {
            JS_SetGCZeal((*js_context).ptr, 0, JS_DEFAULT_ZEAL_FREQ);
        }

        Runtime {
            rt: js_runtime,
            cx: js_context,
        }
    }

    /// Returns the `JSRuntime` object.
    pub fn rt(&self) -> *mut JSRuntime {
        self.rt.ptr
    }

    /// Returns the `JSContext` object.
    pub fn cx(&self) -> *mut JSContext {
        self.cx.ptr
    }
}

pub type rt = rc::Rc<rt_rsrc>;

pub struct rt_rsrc {
    pub ptr : *mut JSRuntime,
}

impl Drop for rt_rsrc {
    fn drop(&mut self) {
        unsafe {
            JS_Finish(self.ptr);
        }
    }
}

pub fn new_runtime(p: *mut JSRuntime) -> rt {
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
            new_context(JS_NewContext(self.ptr,
                                      default_stacksize as size_t), self.clone())
        }
    }
}

pub fn rt() -> rt {
    unsafe {
        let runtime = JS_Init(default_heapsize);
        return new_runtime(runtime);
    }
}

// ___________________________________________________________________________
// contexts

pub struct Cx {
    pub ptr: *mut JSContext,
    pub rt: rt,
}

impl Drop for Cx {
    fn drop(&mut self) {
        unsafe {
            JS_DestroyContext(self.ptr);
        }
    }
}

pub fn new_context(ptr: *mut JSContext, rt: rt) -> rc::Rc<Cx> {
    return rc::Rc::new(Cx {
        ptr: ptr,
        rt: rt,
    })
}

impl Cx {
    pub fn set_default_options_and_version(&self) {
        self.set_options(JSOPTION_VAROBJFIX | JSOPTION_METHODJIT |
                         JSOPTION_TYPE_INFERENCE |
                         JSOPTION_DONT_REPORT_UNCAUGHT |
                         JSOPTION_AUTOJSAPI_OWNS_ERROR_REPORTING);
        self.set_version(JSVERSION_LATEST);
    }

    pub fn set_options(&self, v: c_uint) {
        unsafe {
            JS_SetOptions(self.ptr, v);
        }
    }

    pub fn set_version(&self, v: JSVersion) {
        unsafe {
            JS_SetVersion(self.ptr, v);
        }
    }

    pub fn set_logging_error_reporter(&self) {
        unsafe {
            JS_SetErrorReporter(self.ptr, Some(reportError as unsafe extern "C" fn(*mut JSContext, *const c_char, *mut JSErrorReport)));
        }
    }

    pub fn set_error_reporter(&self, reportfn: unsafe extern "C" fn(*mut JSContext, *const c_char, *mut JSErrorReport)) {
        unsafe {
            JS_SetErrorReporter(self.ptr, Some(reportfn));
        }
    }

    pub fn evaluate_script(&self, glob: *mut JSObject, script: String, filename: String, line_num: usize)
                    -> Result<(),()> {
        let script_utf16: Vec<u16> = script.utf16_units().collect();
        let filename_cstr = ffi::CString::new(filename.as_bytes()).unwrap();
        let mut rval: JSVal = NullValue();
        debug!("Evaluating script from {} with content {}", filename, script);
        // SpiderMonkey does not approve of null pointers.
        let (ptr, len) = if script_utf16.len() == 0 {
            static empty: &'static [u16] = &[];
            (empty.as_ptr(), 0)
        } else {
            (script_utf16.as_ptr(), script_utf16.len() as c_uint)
        };
        assert!(!ptr.is_null());
        unsafe {
            if ERR == JS_EvaluateUCScript(self.ptr, glob, ptr, len,
                                          filename_cstr.as_ptr(), line_num as c_uint,
                                          &mut rval) {
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

pub fn with_compartment<R, F: FnMut() -> R>(cx: *mut JSContext, object: *mut JSObject, mut cb: F) -> R {
    unsafe {
        let call = JS_EnterCrossCompartmentCall(cx, object);
        let result = cb();
        JS_LeaveCrossCompartmentCall(call);
        result
    }
}

#[cfg(test)]
pub mod test {
    use super::rt;
    use super::RtUtils;

    #[test]
    pub fn dummy() {
        let rt = rt();
        let cx = rt.cx();
        cx.deref().set_default_options_and_version();
        cx.deref().set_logging_error_reporter();
    }

}
