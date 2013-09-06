/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/. */

#[doc = "Rust wrappers around the raw JS apis"];

use std::libc::types::os::arch::c95::{size_t, c_uint};
use std::libc::{c_char, uintptr_t};
use std::hashmap::HashMap;
use jsapi::*;
use default_stacksize;
use default_heapsize;
use JSOPTION_VAROBJFIX;
use JSOPTION_METHODJIT;
use JSOPTION_TYPE_INFERENCE;
use JSVAL_NULL;
use ERR;
use name_pool::*;
use global::global_class;
use std::ptr;
use std::ptr::null;
use result;
use result_obj;
use std::uint;
use std::str::raw::from_c_str;
use std::cast;

// ___________________________________________________________________________
// friendly Rustic API to runtimes

pub type rt = @rt_rsrc;

pub struct rt_rsrc {
    ptr : *JSRuntime,
}

impl Drop for rt_rsrc {
    #[fixed_stack_segment]
    fn drop(&self) {
        unsafe {
            JS_Finish(self.ptr);
        }
    }
}

pub fn new_runtime(p: *JSRuntime) -> rt {
    return @rt_rsrc {
        ptr: p
    }
}

impl rt_rsrc {
    #[fixed_stack_segment]
    pub fn cx(@self) -> @Cx {
        unsafe {
            new_context(JS_NewContext(self.ptr, default_stacksize as size_t), self)
        }
    }
}

// FIXME: Is this safe once we have more than one stack segment?
extern fn gc_callback(rt: *JSRuntime, _status: JSGCStatus) {
    use std::rt::local::Local;
    use std::rt::task::Task;
    unsafe {
        do Local::borrow |task: &mut Task| {
            match task.coroutine {
                Some(ref c) => {
                    let start = c.current_stack_segment.start() as uintptr_t;
                    let end = c.current_stack_segment.end() as uintptr_t;
                    JS_SetNativeStackBounds(rt, uint::min(start, end), uint::max(start, end));
                }
                None => {
                    fail!("gc_callback: cannot get stack for JS_SetNativeStackBounds");
                }
            }
        }
    }
}

#[fixed_stack_segment]
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
    classes: @mut HashMap<~str, @JSClass>,
}

#[unsafe_destructor]
impl Drop for Cx {
    #[fixed_stack_segment]
    fn drop(&self) {
        unsafe {
            JS_DestroyContext(self.ptr);
        }
    }
}

pub fn new_context(ptr: *JSContext, rt: rt) -> @Cx {
    return @Cx {
        ptr: ptr,
        rt: rt,
        classes: @mut HashMap::new()
    }
}
    
impl Cx {
    #[fixed_stack_segment]
    pub fn rooted_obj(@self, obj: *JSObject) -> jsobj {
        let jsobj = @jsobj_rsrc {cx: self, cxptr: self.ptr, ptr: obj};
        unsafe {
            JS_AddObjectRoot(self.ptr, ptr::to_unsafe_ptr(&jsobj.ptr));
        }
        jsobj
    }

    pub fn set_default_options_and_version(@self) {
        self.set_options(JSOPTION_VAROBJFIX | JSOPTION_METHODJIT |
                         JSOPTION_TYPE_INFERENCE);
        self.set_version(JSVERSION_LATEST);
    }

    #[fixed_stack_segment]
    pub fn set_options(@self, v: c_uint) {
        unsafe {
            JS_SetOptions(self.ptr, v);
        }
    }

    #[fixed_stack_segment]
    pub fn set_version(@self, v: i32) {
        unsafe {
            JS_SetVersion(self.ptr, v);
        }
    }

    #[fixed_stack_segment]
    pub fn set_logging_error_reporter(@self) {
        unsafe {
            JS_SetErrorReporter(self.ptr, reportError);
        }
    }

    #[fixed_stack_segment]
    pub fn set_error_reporter(@self, reportfn: extern "C" fn(*JSContext, *c_char, *JSErrorReport)) {
        unsafe {
            JS_SetErrorReporter(self.ptr, reportfn);
        }
    }

    #[fixed_stack_segment]
    pub fn new_compartment(@self,
                       globclsfn: &fn(@mut NamePool) -> JSClass)
                    -> Result<@mut Compartment,()> {
        unsafe {
            let np = NamePool();
            let globcls = @globclsfn(np);
            let globobj = JS_NewGlobalObject(self.ptr, ptr::to_unsafe_ptr(&*globcls), null());
            result(JS_InitStandardClasses(self.ptr, globobj)).chain(|_ok| {
                let compartment = @mut Compartment {
                    cx: self,
                    name_pool: np,
                    global_funcs: ~[],
                    global_props: ~[],
                    global_class: globcls,
                    global_obj: self.rooted_obj(globobj),
                    global_protos: @mut HashMap::new()
                };
                self.set_cx_private(ptr::to_unsafe_ptr(&*compartment) as *());
                Ok(compartment)
            })
        }
    }

    #[fixed_stack_segment]
    pub fn new_compartment_with_global(@self, global: *JSObject) -> Result<@mut Compartment,()> {
        let np = NamePool();
        let compartment = @mut Compartment {
            cx: self,
            name_pool: np,
            global_funcs: ~[],
            global_props: ~[],
            global_class: @global_class(np),
            global_obj: self.rooted_obj(global),
            global_protos: @mut HashMap::new()
        };
        unsafe {
            self.set_cx_private(ptr::to_unsafe_ptr(&*compartment) as *());
        }
        Ok(compartment)
    }

    #[fixed_stack_segment]
    pub fn evaluate_script(@self, glob: jsobj, bytes: ~[u8], filename: ~str, line_num: uint) 
                    -> Result<(),()> {
        do bytes.as_imm_buf |bytes_ptr, bytes_len| {
            do filename.to_c_str().with_ref |filename_cstr| {
                let bytes_ptr = bytes_ptr as *c_char;
                let rval: JSVal = JSVAL_NULL;
                debug!("Evaluating script from %s with bytes %?", filename, bytes);
                unsafe {
                    if JS_EvaluateScript(self.ptr, glob.ptr,
                                         bytes_ptr, bytes_len as c_uint,
                                         filename_cstr, line_num as c_uint,
                                         ptr::to_unsafe_ptr(&rval)) == ERR {
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
    }

    pub fn lookup_class_name(@self, s: ~str) ->  @JSClass {
        // FIXME: expect should really take a lambda...
        let error_msg = fmt!("class %s not found in class table", s);
        let name = self.classes.find(&s);
        *(name.expect(error_msg))
    }

    #[fixed_stack_segment]
    pub unsafe fn get_cx_private(@self) -> *() {
        cast::transmute(JS_GetContextPrivate(self.ptr))
    }

    #[fixed_stack_segment]
    pub unsafe fn set_cx_private(@self, data: *()) {
        JS_SetContextPrivate(self.ptr, cast::transmute(data));
    }

    #[fixed_stack_segment]
    pub unsafe fn get_obj_private(@self, obj: *JSObject) -> *() {
        cast::transmute(JS_GetPrivate(obj))
    }

    #[fixed_stack_segment]
    pub unsafe fn set_obj_private(@self, obj: *JSObject, data: *()) {
        JS_SetPrivate(obj, cast::transmute(data));
    }
}

pub extern fn reportError(_cx: *JSContext, msg: *c_char, report: *JSErrorReport) {
    unsafe {
        let fnptr = (*report).filename;
        let fname = if fnptr.is_not_null() {from_c_str(fnptr)} else {~"none"};
        let lineno = (*report).lineno;
        let msg = from_c_str(msg);
        error!("Error at %s:%?: %s\n", fname, lineno, msg);
    }
}

// ___________________________________________________________________________
// compartment

pub struct Compartment {
    cx: @Cx,
    name_pool: @mut NamePool,
    global_funcs: ~[@~[JSFunctionSpec]],
    global_props: ~[@~[JSPropertySpec]],
    global_class: @JSClass,
    global_obj: jsobj,
    global_protos: @mut HashMap<~str, jsobj>
}

impl Compartment {
    #[fixed_stack_segment]
    pub fn define_functions(@mut self,
                        specfn: &fn(@mut NamePool) -> ~[JSFunctionSpec])
                     -> Result<(),()> {
        let specvec = @specfn(self.name_pool);
        self.global_funcs.push(specvec);
        do specvec.as_imm_buf |specs, _len| {
            unsafe {
                result(JS_DefineFunctions(self.cx.ptr, self.global_obj.ptr, specs))
            }
        }
    }
    #[fixed_stack_segment]
    pub fn define_properties(@mut self, specfn: &fn() -> ~[JSPropertySpec]) -> Result<(),()> {
        let specvec = @specfn();
        self.global_props.push(specvec);
        do specvec.as_imm_buf |specs, _len| {
            unsafe {
                result(JS_DefineProperties(self.cx.ptr, self.global_obj.ptr, specs))
            }
        }
    }
    #[fixed_stack_segment]
    pub fn define_property(@mut self,
                       name: ~str,
                       value: JSVal,
                       getter: JSPropertyOp, setter: JSStrictPropertyOp,
                       attrs: c_uint)
                    -> Result<(),()> {
        unsafe {
            result(JS_DefineProperty(self.cx.ptr,
                                     self.global_obj.ptr,
                                     self.add_name(name),
                                     value,
                                     Some(getter),
                                     Some(setter),
                                     attrs))
        }
    }
    #[fixed_stack_segment]
    pub fn new_object(@mut self, class_name: ~str, proto: *JSObject, parent: *JSObject)
               -> Result<jsobj, ()> {
        unsafe {
            let classptr = self.cx.lookup_class_name(class_name);
            let obj = self.cx.rooted_obj(JS_NewObject(self.cx.ptr, &*classptr, proto, parent));
            result_obj(obj)
        }
    }
    #[fixed_stack_segment]
    pub fn new_object_with_proto(@mut self, class_name: ~str, proto_name: ~str, parent: *JSObject)
                          -> Result<jsobj, ()> {
        let classptr = self.cx.lookup_class_name(class_name);
        let proto = self.global_protos.find(&proto_name.clone()).expect(
            fmt!("new_object_with_proto: expected to find %s in the proto \
                  table", proto_name));
        unsafe {
            let obj = self.cx.rooted_obj(JS_NewObject(self.cx.ptr, ptr::to_unsafe_ptr(&*classptr),
                                                      proto.ptr, parent));
            result_obj(obj)
        }
    }
    pub fn get_global_proto(@mut self, name: ~str) -> jsobj {
        let proto = self.global_protos.get(&name);
        *proto
    }
    pub fn stash_global_proto(@mut self, name: ~str, proto: jsobj) {
        let global_protos = self.global_protos;
        if !global_protos.insert(name, proto) {
            fail!(~"Duplicate global prototype registered; you're gonna have a bad time.")
        }
    }
    pub fn register_class(@mut self, class_fn: &fn(x: @mut Compartment) -> JSClass) {
        let classptr = @class_fn(self);
        if !self.cx.classes.insert(
            unsafe { from_c_str(classptr.name) },
            classptr) {
            fail!(~"Duplicate JSClass registered; you're gonna have a bad time.")
        }
    }
    pub fn add_name(@mut self, name: ~str) -> *c_char {
        self.name_pool.add(name.clone())
    }
}

// ___________________________________________________________________________
// objects

pub type jsobj = @jsobj_rsrc;

pub struct jsobj_rsrc {
    cx: @Cx,
    cxptr: *JSContext,
    ptr: *JSObject,
}

#[unsafe_destructor]
impl Drop for jsobj_rsrc {
    #[fixed_stack_segment]
    fn drop(&self) {
        unsafe {
            JS_RemoveObjectRoot(self.cxptr, ptr::to_unsafe_ptr(&self.ptr));
        }
    }
}

impl jsobj_rsrc {
    fn new_object(&self, cx: @Cx, cxptr: *JSContext, ptr: *JSObject) -> jsobj {
        return @jsobj_rsrc {
            cx: cx,
            cxptr: cxptr,
            ptr: ptr
        }
    }
}

// ___________________________________________________________________________
// random utilities

pub trait to_jsstr {
    fn to_jsstr(self, cx: @Cx) -> *JSString;
}

impl to_jsstr for ~str {
    #[fixed_stack_segment]
    fn to_jsstr(self, cx: @Cx) -> *JSString {
        do self.as_imm_buf |buf, len| {
            unsafe {
                let cbuf = cast::transmute(buf);
                JS_NewStringCopyN(cx.ptr, cbuf, len as size_t)
            }
        }
    }
}

#[cfg(test)]
pub mod test {
    use super::rt;
    use super::super::global;
    use super::super::jsapi::{JS_GC, JS_GetRuntime};

    #[test]
    #[fixed_stack_segment]
    pub fn dummy() {
        let rt = rt();
        let cx = rt.cx();
        cx.set_default_options_and_version();
        cx.set_logging_error_reporter();
        cx.new_compartment(global::global_class).chain(|comp| {
            unsafe { JS_GC(JS_GetRuntime(comp.cx.ptr)); }

            comp.define_functions(global::debug_fns);

            let s = ~"debug(22);";
            let bytes = s.as_bytes().to_owned();
            cx.evaluate_script(comp.global_obj, bytes, ~"test", 1u)
        });
    }

}
