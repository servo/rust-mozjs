#[doc = "Rust wrappers around the raw JS apis"];

import bg = jsapi::bindgen;
import libc::types::os::arch::c95::{size_t, c_uint};
import std::map::{hashmap, str_hash};

export rt;
export cx;
export jsobj;
export methods;
export compartment;

// ___________________________________________________________________________
// friendly Rustic API to runtimes

type rt = @rt_rsrc;

struct rt_rsrc {
    let ptr : *JSRuntime;
    new(p : {ptr: *JSRuntime}) {
        self.ptr = p.ptr;
    }
    drop {
        JS_Finish(self.ptr);
    }
}

fn rt() -> rt {
    @rt_rsrc({ptr: JS_Init(default_heapsize)})
}

impl rt {
    fn cx() -> cx {
        @cx_rsrc({ptr: JS_NewContext(self.ptr, default_stacksize as size_t),
                  rt: self})
    }
}

// ___________________________________________________________________________
// contexts

type cx = @cx_rsrc;

struct cx_rsrc {
    let ptr : *JSContext;
    let rt: rt;
    let classes: hashmap<~str, @JSClass>;

    new(rec : {ptr: *JSContext, rt: rt}) {
        self.ptr = rec.ptr;
        self.rt = rec.rt;
        self.classes = str_hash();
    }
    drop {
        JS_DestroyContext(self.ptr);
    }
}
    
impl cx {
    fn rooted_obj(obj: *JSObject) -> jsobj {
        let jsobj = @jsobj_rsrc({cx: self, cxptr: self.ptr, ptr: obj});
        JS_AddObjectRoot(self.ptr, ptr::addr_of(jsobj.ptr));
        jsobj
    }

    fn set_default_options_and_version() {
        self.set_options(JSOPTION_VAROBJFIX | JSOPTION_METHODJIT);
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

    fn new_compartment(globclsfn: fn(name_pool) -> JSClass) -> result<compartment,()> {
        let np = name_pool();
        let globcls = @globclsfn(np);
        let globobj = JS_NewCompartmentAndGlobalObject(self.ptr, ptr::assimilate(&*globcls), null());
        result(JS_InitStandardClasses(self.ptr, globobj)).chain(|_ok| {
            let compartment = @{cx: self,
                                name_pool: np,
                                mut global_funcs: ~[],
                                mut global_props: ~[],
                                global_class: globcls,
                                global_obj: self.rooted_obj(globobj),
                                global_protos: str_hash()
                               };
            self.set_cx_private(ptr::assimilate(&*compartment) as *());
            ok(compartment)
        })
    }

    fn evaluate_script(glob: jsobj, bytes: ~[u8], filename: ~str, line_num: uint) 
        -> result<(),()> {
        vec::as_buf(bytes, |bytes_ptr, bytes_len| {
            str::as_c_str(filename, |filename_cstr| {
                let bytes_ptr = bytes_ptr as *c_char;
                let v: jsval = 0_u64;
                #debug["Evaluating script from %s with bytes %?", filename, bytes];
                if JS_EvaluateScript(self.ptr, glob.ptr,
                                     bytes_ptr, bytes_len as c_uint,
                                     filename_cstr, line_num as c_uint,
                                     ptr::addr_of(v)) == ERR {
                    #debug["...err!"];
                    err(())
                } else {
                    // we could return the script result but then we'd have
                    // to root it and so forth and, really, who cares?
                    #debug["...ok!"];
                    ok(())
                }
            })
        })
    }

    fn lookup_class_name(s: ~str) ->  @JSClass {
      option::expect(self.classes.find(s),
           #fmt("Class %s not found in class table", s))
    }

    unsafe fn get_cx_private() -> *() {
        unsafe::reinterpret_cast(JS_GetContextPrivate(self.ptr))
    }

    unsafe fn set_cx_private(data: *()) {
        JS_SetContextPrivate(self.ptr, unsafe::reinterpret_cast(data));
    }

    unsafe fn get_obj_private(obj: *JSObject) -> *() {
        unsafe::reinterpret_cast(JS_GetPrivate(obj))
    }

    unsafe fn set_obj_private(obj: *JSObject, data: *()) {
        JS_SetPrivate(obj, unsafe::reinterpret_cast(data));
    }
}

extern fn reportError(_cx: *JSContext,
                     msg: *c_char,
                     report: *JSErrorReport) {
    unsafe {
        let fnptr = (*report).filename;
        let fname = if fnptr.is_not_null() {from_c_str(fnptr)} else {~"none"};
        let lineno = (*report).lineno;
        let msg = from_c_str(msg);
        #error["Error at %s:%?: %s\n", fname, lineno, msg];
    }
}

// ___________________________________________________________________________
// compartment

type bare_compartment = {
    cx: cx,
    name_pool: name_pool,
    mut global_funcs: ~[@~[JSFunctionSpec]],
    mut global_props: ~[@~[JSPropertySpec]],
    global_class: @JSClass,
    global_obj: jsobj,
    global_protos: hashmap<~str, jsobj>
};

trait methods {
    fn define_functions(specfn: fn(name_pool) -> ~[JSFunctionSpec]) -> result<(),()>;
    fn define_properties(specfn: fn() -> ~[JSPropertySpec]) -> result<(),()>;
    fn define_property(name: ~str, value: jsval, getter: JSPropertyOp,
                       setter: JSStrictPropertyOp, attrs: c_uint) -> result<(),()>;
    fn new_object(class_name: ~str, proto: *JSObject, parent: *JSObject)
        -> result<jsobj, ()>;
    fn new_object_with_proto(class_name: ~str, proto_name: ~str, parent: *JSObject)
        -> result<jsobj, ()>;
    fn register_class(class_fn: fn(bare_compartment) -> JSClass);
    fn get_global_proto(name: ~str) -> jsobj;
    fn stash_global_proto(name: ~str, proto: jsobj);
    fn add_name(name: ~str) -> *c_char;
}

type compartment = @bare_compartment;

impl bare_compartment : methods {
    fn define_functions(specfn: fn(name_pool) -> ~[JSFunctionSpec]) -> result<(),()> {
        let specvec = @specfn(self.name_pool);
        vec::push(self.global_funcs, specvec);
        vec::as_buf(*specvec, |specs, _len| {
            result(JS_DefineFunctions(self.cx.ptr, self.global_obj.ptr, specs))
        })
    }
    fn define_properties(specfn: fn() -> ~[JSPropertySpec]) -> result<(),()> {
        let specvec = @specfn();
        vec::push(self.global_props, specvec);
        vec::as_buf(*specvec, |specs, _len| {
            result(JS_DefineProperties(self.cx.ptr, self.global_obj.ptr, specs))
        })
    }
    fn define_property(name: ~str, value: jsval, getter: JSPropertyOp,
                       setter: JSStrictPropertyOp, attrs: c_uint) -> result<(),()> {
        result(JS_DefineProperty(self.cx.ptr, self.global_obj.ptr, self.add_name(name),
                                 value, getter, setter, attrs))
    }
    fn new_object(class_name: ~str, proto: *JSObject, parent: *JSObject)
        -> result<jsobj, ()> {
        let classptr = self.cx.lookup_class_name(class_name);
        let obj = self.cx.rooted_obj(JS_NewObject(self.cx.ptr, ptr::assimilate(&*classptr),
                                                  proto, parent));
        result_obj(obj)
    }
    fn new_object_with_proto(class_name: ~str, proto_name: ~str, parent: *JSObject)
        -> result<jsobj, ()> {
        let classptr = self.cx.lookup_class_name(class_name);
        let proto = option::expect(self.global_protos.find(proto_name),
           #fmt("new_object_with_proto: expected to find %s in the proto \
              table", proto_name));
        let obj = self.cx.rooted_obj(JS_NewObject(self.cx.ptr, ptr::assimilate(&*classptr),
                                                  proto.ptr, parent));
        result_obj(obj)
    }
    fn get_global_proto(name: ~str) -> jsobj {
        self.global_protos.get(name)
    }
    fn stash_global_proto(name: ~str, proto: jsobj) {
        if !self.global_protos.insert(name, proto) {
            fail ~"Duplicate global prototype registered; you're gonna have a bad time."
        }
    }
    fn register_class(class_fn: fn(bare_compartment) -> JSClass) {
        let classptr = @class_fn(self);
        if !self.cx.classes.insert(
            unsafe { str::unsafe::from_c_str(classptr.name) },
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

type jsobj = @jsobj_rsrc;

struct jsobj_rsrc {
    let cx : cx;
    let cxptr : *JSContext;
    let ptr : *JSObject;
    new(rec : {cx: cx, cxptr: *JSContext, ptr: *JSObject}) {
        self.cx = rec.cx;
        self.cxptr = rec.cxptr;
        self.ptr = rec.ptr;
    }
    drop {
        JS_RemoveObjectRoot(self.cxptr, ptr::addr_of(self.ptr));
    }
}

// ___________________________________________________________________________
// random utilities

trait to_jsstr {
    fn to_jsstr(cx: cx) -> *JSString;
}

impl ~str : to_jsstr {
    fn to_jsstr(cx: cx) -> *JSString {
        str::as_buf(self, |buf, len| {
            let cbuf = unsafe { unsafe::reinterpret_cast(buf) };
            bg::JS_NewStringCopyN(cx.ptr, cbuf, len as size_t)
        })
    }
}

#[cfg(test)]
mod test {

    #[test]
    fn dummy() {
        let rt = rt();
        let cx = rt.cx();
        cx.set_default_options_and_version();
        cx.set_logging_error_reporter();
        cx.new_compartment(global::global_class).chain(|comp| {
            comp.define_functions(global::debug_fns);

            let bytes = str::bytes(~"debug(22);");
            cx.evaluate_script(comp.global_obj, bytes, ~"test", 1u)
        });
    }

}
