#[doc = "Rust wrappers around the raw JS apis"];

#[allow(non_implicitly_copyable_typarams)];

use bg = jsapi::bindgen;
use core::libc::types::os::arch::c95::{size_t, c_uint};
use core::libc::c_char;
use std::oldmap::HashMap;
use jsapi::*;
use jsapi::bindgen::*;
use default_stacksize;
use default_heapsize;
use JSOPTION_VAROBJFIX;
use JSOPTION_METHODJIT;
use JSOPTION_TYPE_INFERENCE;
use JSVAL_NULL;
use ERR;
use name_pool::*;
use core::ptr::null;
use result;
use result_obj;
use core::str::raw::from_c_str;

// ___________________________________________________________________________
// friendly Rustic API to runtimes

pub type rt = @rt_rsrc;

pub struct rt_rsrc {
    ptr : *JSRuntime,
}

impl Drop for rt_rsrc {
    fn finalize(&self) {
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

pub impl rt {
    fn cx(&self) -> @Cx {
        unsafe {
            new_context(JS_NewContext(self.ptr, default_stacksize as size_t), *self)
        }
    }
}


pub fn rt() -> rt {
    unsafe {
        return new_runtime(JS_Init(default_heapsize))
    }
}

// ___________________________________________________________________________
// contexts

pub struct Cx {
    ptr: *JSContext,
    rt: rt,
    classes: HashMap<~str, @JSClass>,
}

impl Drop for Cx {
    fn finalize(&self) {
        unsafe {
            JS_DestroyContext(self.ptr);
        }
    }
}

pub fn new_context(ptr: *JSContext, rt: rt) -> @Cx {
    return @Cx {
        ptr: ptr,
        rt: rt,
        classes: HashMap()
    }
}
    
pub impl Cx {
    fn rooted_obj(@self, obj: *JSObject) -> jsobj {
        let jsobj = @jsobj_rsrc {cx: self, cxptr: self.ptr, ptr: obj};
        unsafe {
            JS_AddObjectRoot(self.ptr, ptr::to_unsafe_ptr(&jsobj.ptr));
        }
        jsobj
    }

    fn set_default_options_and_version(@self) {
        self.set_options(JSOPTION_VAROBJFIX | JSOPTION_METHODJIT |
                         JSOPTION_TYPE_INFERENCE);
        self.set_version(JSVERSION_LATEST);
    }

    fn set_options(@self, v: c_uint) {
        unsafe {
            JS_SetOptions(self.ptr, v);
        }
    }

    fn set_version(@self, v: i32) {
        unsafe {
            JS_SetVersion(self.ptr, v);
        }
    }

    fn set_logging_error_reporter(@self) {
        unsafe {
            JS_SetErrorReporter(self.ptr, reportError);
        }
    }

    fn set_error_reporter(@self, reportfn: *u8) {
        unsafe {
            JS_SetErrorReporter(self.ptr, reportfn);
        }
    }

    fn new_compartment(@self,
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
                    global_protos: HashMap()
                };
                self.set_cx_private(ptr::to_unsafe_ptr(&*compartment) as *());
                Ok(compartment)
            })
        }
    }

    fn evaluate_script(@self, glob: jsobj, bytes: ~[u8], filename: ~str, line_num: uint) 
                    -> Result<(),()> {
        vec::as_imm_buf(bytes, |bytes_ptr, bytes_len| {
            str::as_c_str(filename, |filename_cstr| {
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
            })
        })
    }

    fn lookup_class_name(@self, s: ~str) ->  @JSClass {
        // FIXME: expect should really take a lambda...
        let error_msg = fmt!("class %s not found in class table", s);
        option::expect(self.classes.find(&s), error_msg)
    }

    unsafe fn get_cx_private(@self) -> *() {
        cast::reinterpret_cast(&JS_GetContextPrivate(self.ptr))
    }

    unsafe fn set_cx_private(@self, data: *()) {
        JS_SetContextPrivate(self.ptr, cast::reinterpret_cast(&data));
    }

    unsafe fn get_obj_private(@self, obj: *JSObject) -> *() {
        cast::reinterpret_cast(&JS_GetPrivate(obj))
    }

    unsafe fn set_obj_private(@self, obj: *JSObject, data: *()) {
        JS_SetPrivate(obj, cast::reinterpret_cast(&data));
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
    global_protos: HashMap<~str, jsobj>
}

pub impl Compartment {
    fn define_functions(@mut self,
                        specfn: &fn(@mut NamePool) -> ~[JSFunctionSpec])
                     -> Result<(),()> {
        let specvec = @specfn(self.name_pool);
        vec::push(&mut self.global_funcs, specvec);
        vec::as_imm_buf(*specvec, |specs, _len| {
            unsafe {
                result(JS_DefineFunctions(self.cx.ptr, self.global_obj.ptr, specs))
            }
        })
    }
    fn define_properties(@mut self, specfn: &fn() -> ~[JSPropertySpec]) -> Result<(),()> {
        let specvec = @specfn();
        vec::push(&mut self.global_props, specvec);
        vec::as_imm_buf(*specvec, |specs, _len| {
            unsafe {
                result(JS_DefineProperties(self.cx.ptr, self.global_obj.ptr, specs))
            }
        })
    }
    fn define_property(@mut self,
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
                                     getter,
                                     setter,
                                     attrs))
        }
    }
    fn new_object(@mut self, class_name: ~str, proto: *JSObject, parent: *JSObject)
               -> Result<jsobj, ()> {
        unsafe {
            let classptr = self.cx.lookup_class_name(class_name);
            let obj = self.cx.rooted_obj(JS_NewObject(self.cx.ptr, &*classptr, proto, parent));
            result_obj(obj)
        }
    }
    fn new_object_with_proto(@mut self, class_name: ~str, proto_name: ~str, parent: *JSObject)
                          -> Result<jsobj, ()> {
        let classptr = self.cx.lookup_class_name(class_name);
        let proto = option::expect(self.global_protos.find(&copy proto_name),
           fmt!("new_object_with_proto: expected to find %s in the proto \
              table", proto_name));
        unsafe {
            let obj = self.cx.rooted_obj(JS_NewObject(self.cx.ptr, ptr::to_unsafe_ptr(&*classptr),
                                                      proto.ptr, parent));
            result_obj(obj)
        }
    }
    fn get_global_proto(@mut self, name: ~str) -> jsobj {
        self.global_protos.get(&name)
    }
    fn stash_global_proto(@mut self, name: ~str, proto: jsobj) {
        let global_protos = self.global_protos;
        if !global_protos.insert(name, proto) {
            fail!(~"Duplicate global prototype registered; you're gonna have a bad time.")
        }
    }
    fn register_class(@mut self, class_fn: &fn(x: @mut Compartment) -> JSClass) {
        let classptr = @class_fn(self);
        if !self.cx.classes.insert(
            unsafe { from_c_str(classptr.name) },
            classptr) {
            fail!(~"Duplicate JSClass registered; you're gonna have a bad time.")
        }
    }
    fn add_name(@mut self, name: ~str) -> *c_char {
        self.name_pool.add(copy name)
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

impl Drop for jsobj_rsrc {
    fn finalize(&self) {
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
    fn to_jsstr(self, cx: @Cx) -> *JSString {
        str::as_buf(self, |buf, len| {
            unsafe {
                let cbuf = cast::reinterpret_cast(&buf);
                bg::JS_NewStringCopyN(cx.ptr, cbuf, len as size_t)
            }
        })
    }
}

#[cfg(test)]
pub mod test {
    use rt;
    use global;

    #[test]
    pub fn dummy() {
        let rt = rt();
        let cx = rt.cx();
        cx.set_default_options_and_version();
        cx.set_logging_error_reporter();
        cx.new_compartment(global::global_class).chain(|comp| {
            comp.define_functions(global::debug_fns);

            let bytes = str::to_bytes(~"debug(22);");
            cx.evaluate_script(comp.global_obj, bytes, ~"test", 1u)
        });
    }

}
