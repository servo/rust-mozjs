/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Rust wrappers around the raw JS apis

use libc::types::os::arch::c95::{size_t, c_uint};
use libc::uintptr_t;
use libc::c_char;
use std::cmp;
use std::rc;
use std::rt::Runtime;
use std::string;
use jsapi::*;
use jsval::{JSVal, NullValue};
use default_stacksize;
use default_heapsize;
use JSOPTION_VAROBJFIX;
use JSOPTION_METHODJIT;
use JSOPTION_TYPE_INFERENCE;
use ERR;

// ___________________________________________________________________________
// friendly Rustic API to runtimes

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
            new_context(JS_NewContext(self.deref().ptr,
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

#[unsafe_destructor]
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
                         JSOPTION_TYPE_INFERENCE);
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
            JS_SetErrorReporter(self.ptr, Some(reportError));
        }
    }

    pub fn set_error_reporter(&self, reportfn: unsafe extern "C" fn(*mut JSContext, *const c_char, *mut JSErrorReport)) {
        unsafe {
            JS_SetErrorReporter(self.ptr, Some(reportfn));
        }
    }

    pub fn evaluate_script(&self, glob: *mut JSObject, script: String, filename: String, line_num: uint)
                    -> Result<(),()> {
        let script_utf16: Vec<u16> = script.as_slice().utf16_units().collect();
        let filename_cstr = filename.to_c_str();
        let mut rval: JSVal = NullValue();
        debug!("Evaluating script from {:s} with content {}", filename, script);
        // SpiderMonkey does not approve of null pointers.
        let (ptr, len) = if script_utf16.len() == 0 {
            static empty: &'static [u16] = &[];
            (empty.as_ptr(), 0)
        } else {
            (script_utf16.as_ptr(), script_utf16.len() as c_uint)
        };
        assert!(ptr.is_not_null());
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
    let fname = if fnptr.is_not_null() {string::raw::from_buf(fnptr as *const i8 as *const u8)} else {"none".to_string()};
    let lineno = (*report).lineno;
    let msg = string::raw::from_buf(msg as *const i8 as *const u8);
    error!("Error at {:s}:{}: {:s}\n", fname, lineno, msg);
}

pub fn with_compartment<R>(cx: *mut JSContext, object: *mut JSObject, cb: || -> R) -> R {
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
