#[doc = "Rust wrappers around the raw JS apis"];

use bg = jsapi::bindgen;
use libc::types::os::arch::c95::{size_t, c_uint};
use std::map::HashMap;

export rt;
export cx;
export jsobj;
export methods;
export compartment;

// ___________________________________________________________________________
// friendly Rustic API to runtimes

pub type rt = @rt_rsrc;

pub struct rt_rsrc {
    ptr : *JSRuntime,
    drop {
        JS_Finish(self.ptr);
    }
}

pub fn new_runtime(p : {ptr: *JSRuntime}) -> rt {
    return @rt_rsrc {
        ptr: p.ptr 
    }
}

impl rt {
    fn cx() -> cx {
        new_context({ ptr: JS_NewContext(self.ptr, default_stacksize as size_t),
                      rt: self})
    }
}


pub fn rt() -> rt {
    return new_runtime({ptr: JS_Init(default_heapsize)})
}

// ___________________________________________________________________________
// contexts

pub type cx = @cx_rsrc;

pub struct cx_rsrc {
    ptr : *JSContext,
    rt: rt,
    classes: HashMap<~str, @JSClass>,

    drop {
        JS_DestroyContext(self.ptr);
    }
}

pub fn new_context(rec : {ptr: *JSContext, rt: rt}) -> cx {
    return @cx_rsrc {
        ptr: rec.ptr,
        rt: rec.rt,
        classes: HashMap()
    }
}
    
impl cx {
    fn rooted_obj(obj: *JSObject) -> jsobj {
        let jsobj = @jsobj_rsrc {cx: self, cxptr: self.ptr, ptr: obj};
        JS_AddObjectRoot(self.ptr, ptr::to_unsafe_ptr(&jsobj.ptr));
        jsobj
    }

    fn set_default_options_and_version() {
        self.set_options(JSOPTION_VAROBJFIX | JSOPTION_METHODJIT |
                         JSOPTION_TYPE_INFERENCE);
        self.set_version(JSVERSION_LATEST);
    }

    fn set_options(v: c_uint) {
        JS_SetOptions(self.ptr, v);
    }

    fn set_version(v: i32) {
        JS_SetVersion(self.ptr, v);
    }

    fn set_logging_error_reporter() {
        JS_SetErrorReporter(self.ptr, reportError);
    }

    fn set_error_reporter(reportfn: *u8) {
        JS_SetErrorReporter(self.ptr, reportfn);
    }

    fn new_compartment(globclsfn: fn(NamePool) -> JSClass) -> Result<compartment,()> {
        let np = NamePool();
        let globcls = @globclsfn(np);
        let globobj = JS_NewGlobalObject(self.ptr, ptr::to_unsafe_ptr(&*globcls), null());
        result(JS_InitStandardClasses(self.ptr, globobj)).chain(|_ok| {
            let compartment = @{cx: self,
                                name_pool: np,
                                mut global_funcs: ~[],
                                mut global_props: ~[],
                                global_class: globcls,
                                global_obj: self.rooted_obj(globobj),
                                global_protos: HashMap()
                               };
            self.set_cx_private(ptr::to_unsafe_ptr(&*compartment) as *());
            Ok(compartment)
        })
    }

    fn evaluate_script(glob: jsobj, bytes: ~[u8], filename: ~str, line_num: uint) 
                    -> Result<(),()> {
        vec::as_imm_buf(bytes, |bytes_ptr, bytes_len| {
            str::as_c_str(filename, |filename_cstr| {
                let bytes_ptr = bytes_ptr as *c_char;
                let rval: JSVal = JSVAL_NULL;
                debug!("Evaluating script from %s with bytes %?", filename, bytes);
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
            })
        })
    }

    fn lookup_class_name(s: ~str) ->  @JSClass {
        // FIXME: expect should really take a lambda...
        let error_msg = fmt!("class %s not found in class table", s);
        option::expect(self.classes.find(move s), move error_msg)
    }

    unsafe fn get_cx_private() -> *() {
        cast::reinterpret_cast(&JS_GetContextPrivate(self.ptr))
    }

    unsafe fn set_cx_private(data: *()) {
        JS_SetContextPrivate(self.ptr, cast::reinterpret_cast(&data));
    }

    unsafe fn get_obj_private(obj: *JSObject) -> *() {
        cast::reinterpret_cast(&JS_GetPrivate(obj))
    }

    unsafe fn set_obj_private(obj: *JSObject, data: *()) {
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

pub type bare_compartment = {
    cx: cx,
    name_pool: NamePool,
    mut global_funcs: ~[@~[JSFunctionSpec]],
    mut global_props: ~[@~[JSPropertySpec]],
    global_class: @JSClass,
    global_obj: jsobj,
    global_protos: HashMap<~str, jsobj>
};

pub trait methods {
    fn define_functions(specfn: fn(NamePool) -> ~[JSFunctionSpec]) -> Result<(),()>;
    fn define_properties(specfn: fn() -> ~[JSPropertySpec]) -> Result<(),()>;
    fn define_property(name: ~str, value: JSVal, getter: JSPropertyOp,
                       setter: JSStrictPropertyOp, attrs: c_uint) -> Result<(),()>;
    fn new_object(class_name: ~str, proto: *JSObject, parent: *JSObject) -> Result<jsobj, ()>;
    fn new_object_with_proto(class_name: ~str, proto_name: ~str, parent: *JSObject)
                          -> Result<jsobj, ()>;
    fn register_class(class_fn: fn(x: &bare_compartment) -> JSClass);
    fn get_global_proto(name: ~str) -> jsobj;
    fn stash_global_proto(name: ~str, proto: jsobj);
    fn add_name(name: ~str) -> *c_char;
}

pub type compartment = @bare_compartment;

impl bare_compartment : methods {
    fn define_functions(specfn: fn(NamePool) -> ~[JSFunctionSpec]) -> Result<(),()> {
        let specvec = @specfn(self.name_pool);
        vec::push(&mut self.global_funcs, specvec);
        vec::as_imm_buf(*specvec, |specs, _len| {
            result(JS_DefineFunctions(self.cx.ptr, self.global_obj.ptr, specs))
        })
    }
    fn define_properties(specfn: fn() -> ~[JSPropertySpec]) -> Result<(),()> {
        let specvec = @specfn();
        vec::push(&mut self.global_props, specvec);
        vec::as_imm_buf(*specvec, |specs, _len| {
            result(JS_DefineProperties(self.cx.ptr, self.global_obj.ptr, specs))
        })
    }
    fn define_property(name: ~str, value: JSVal, getter: JSPropertyOp, setter: JSStrictPropertyOp,
                       attrs: c_uint)
                    -> Result<(),()> {
        result(JS_DefineProperty(self.cx.ptr, self.global_obj.ptr, self.add_name(move name),
                                 value, getter, setter, attrs))
    }
    fn new_object(class_name: ~str, proto: *JSObject, parent: *JSObject)
               -> Result<jsobj, ()> {
        let classptr = self.cx.lookup_class_name(move class_name);
        let obj = self.cx.rooted_obj(JS_NewObject(self.cx.ptr, ptr::to_unsafe_ptr(&*classptr),
                                                  proto, parent));
        result_obj(obj)
    }
    fn new_object_with_proto(class_name: ~str, proto_name: ~str, parent: *JSObject)
                          -> Result<jsobj, ()> {
        let classptr = self.cx.lookup_class_name(move class_name);
        let proto = option::expect(self.global_protos.find(copy proto_name),
           fmt!("new_object_with_proto: expected to find %s in the proto \
              table", proto_name));
        let obj = self.cx.rooted_obj(JS_NewObject(self.cx.ptr, ptr::to_unsafe_ptr(&*classptr),
                                                  proto.ptr, parent));
        result_obj(obj)
    }
    fn get_global_proto(name: ~str) -> jsobj {
        self.global_protos.get(move name)
    }
    fn stash_global_proto(name: ~str, proto: jsobj) {
        if !self.global_protos.insert(move name, move proto) {
            fail ~"Duplicate global prototype registered; you're gonna have a bad time."
        }
    }
    fn register_class(class_fn: fn(x: &bare_compartment) -> JSClass) {
        let classptr = @class_fn(&self);
        if !self.cx.classes.insert(
            unsafe { str::raw::from_c_str(classptr.name) },
            classptr) {
            fail ~"Duplicate JSClass registered; you're gonna have a bad time."
        }
    }
    fn add_name(name: ~str) -> *c_char {
        self.name_pool.add(copy name)
    }
}

// ___________________________________________________________________________
// objects

pub type jsobj = @jsobj_rsrc;

pub struct jsobj_rsrc {
    cx : cx,
    cxptr : *JSContext,
    ptr : *JSObject,
    drop {
        JS_RemoveObjectRoot(self.cxptr, ptr::to_unsafe_ptr(&self.ptr));
    }
}

impl jsobj_rsrc {
    fn new_object(rec : {cx: cx, cxptr: *JSContext, ptr: *JSObject}) -> jsobj {
        return @jsobj_rsrc {
            cx: rec.cx,
            cxptr: rec.cxptr,
            ptr: rec.ptr
        }
    }
}

// ___________________________________________________________________________
// random utilities

pub trait to_jsstr {
    fn to_jsstr(cx: cx) -> *JSString;
}

impl ~str : to_jsstr {
    fn to_jsstr(cx: cx) -> *JSString {
        str::as_buf(self, |buf, len| {
            let cbuf = unsafe { cast::reinterpret_cast(&buf) };
            bg::JS_NewStringCopyN(cx.ptr, cbuf, len as size_t)
        })
    }
}

#[cfg(test)]
pub mod test {

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
