import comm::*;
import libc::*;
import js = jsapi::bindgen;
import jsapi::*;
import ptr::null;
import void = c_void;

type error_report = {
	message: str,
	filename: str,
	lineno: u32,
	flags: u32
};

type log_message = {
	message: str,
	level: u32,
        tag: u32,
        timeout: u32
};

enum object { object_priv(*JSObject) }
enum principals { principals_priv(*JSPrincipals) }
enum script { script_priv(*JSScript) }
enum string { string_priv(*JSString) }

resource runtime(_rt : *JSRuntime) {
    // because there is one runtime per thread, raii does not
    // work. one task will finish but there may be other tasks
    // on the same os thread.
    //js::JS_Finish(rt);
}

resource context(_cx : *JSContext) {
    //js::JS_DestroyContext(cx);
}

fn begin_request(cx : *JSContext) {
    js::JS_BeginRequest(cx);
}

fn end_request(cx : *JSContext) {
    js::JS_EndRequest(cx);
}

resource request(cx : *JSContext) {
    js::JS_EndRequest(cx);
}

/* Runtimes */

fn new_runtime(maxbytes : u32) -> runtime {
    ret runtime(js::JS_Init(maxbytes));
}

fn shut_down() {
    js::JS_ShutDown();
}

/* Contexts */

fn new_context(rt : runtime, stack_chunk_size : size_t) -> context {
    ret context(js::JS_NewContext(*rt, stack_chunk_size));
}

/* Options */

fn get_options(cx : context) -> u32 {
    ret js::JS_GetOptions(*cx);
}

fn set_options(cx : context, options : u32) {
    let _ = js::JS_SetOptions(*cx, options);
}

fn set_version(cx : context, version : JSVersion) {
    let _ = js::JS_SetVersion(*cx, version);
}

/* Objects */

fn new_compartment_and_global_object(cx : context, clas : @class,
                                     principals : principals) -> object {
    let jsclass = ptr::addr_of(clas.jsclass);
    let jsobj = js::JS_NewCompartmentAndGlobalObject(*cx, jsclass,
                                                     *principals);
    if jsobj == null() { fail; }
    ret object_priv(jsobj);
}

/* Principals */

fn null_principals() -> principals {
    ret principals_priv(null());
}

/* Classes */

type class_spec = {
    name: str,
    flags: u32
    /* TODO: More to add here. */
};

type class = {
    name: @str,
    jsclass: JSClass
};

fn new_class(spec : class_spec) -> @class unsafe {
    // Root the name separately, and make the JSClass name point into it.
    let name = @spec.name;
    let x : *void = ptr::null();
    ret @{
        name: name,
        jsclass: {
            name: str::as_c_str(*name, { |b| b }),
            flags: spec.flags,

            addProperty: crust::JS_PropertyStub,
            delProperty: crust::JS_PropertyStub,
            getProperty: crust::JS_PropertyStub,
            setProperty: crust::JS_StrictPropertyStub,
            enumerate: crust::JS_EnumerateStub,
            resolve: crust::JS_ResolveStub,
            convert: crust::JS_ConvertStub,
            finalize: crust::JS_FinalizeStub,

            reserved0: unsafe::reinterpret_cast(0),
            checkAccess: unsafe::reinterpret_cast(0),
            call: unsafe::reinterpret_cast(0),
            construct: unsafe::reinterpret_cast(0),
            xdrObject: unsafe::reinterpret_cast(0),
            hasInstance: unsafe::reinterpret_cast(0),
            trace: unsafe::reinterpret_cast(0),

            reserved1: unsafe::reinterpret_cast(0),
            reserved: (x,x,x,x,x,x,x,x, x,x,x,x,x,x,x,x,    /* 16 */
                       x,x,x,x,x,x,x,x, x,x,x,x,x,x,x,x,    /* 32 */
                       x,x,x,x,x,x,x,x)

        }
    };
}

/* Standard classes */

fn init_standard_classes(cx : context, object : object) {
    if js::JS_InitStandardClasses(*cx, *object) == 0 as JSBool { fail; }
}

/* Script compilation */

fn compile_script(cx : context, object : object, src : [u8], filename : str,
                  lineno : uint) -> script unsafe {
    let jsscript = str::as_c_str(filename, { |buf|
        js::JS_CompileScript(*cx, *object,
                             unsafe::reinterpret_cast(vec::unsafe::to_ptr(src)),
                             vec::len(src) as size_t, buf, lineno as c_uint)
    });
    if jsscript == ptr::null() {
        fail;   // TODO: this is antisocial
    }
    ret script_priv(jsscript);
}

/* Script execution */

fn execute_script(cx : context, object : object, script : script)
        -> option<jsval> unsafe {
    let rv : jsval = unsafe::reinterpret_cast(0);
    if js::JS_ExecuteScript(*cx, *object, *script, ptr::addr_of(rv)) == 0 as JSBool {
        ret none;
    }
    ret some(rv);
}

/* Value conversion */

fn value_to_source(cx : context, v : jsval) -> string {
    ret string_priv(js::JS_ValueToSource(*cx, v));
}

/* String conversion */

fn get_string_bytes(cx : context, jsstr : string) -> [u16] unsafe {
    // FIXME: leaks, probably
    let size = 0 as size_t;
    let bytes = js::JS_GetStringCharsZAndLength(*cx, *jsstr,
                                                ptr::addr_of(size));
    ret vec::unsafe::from_buf(bytes, ((size + (1 as size_t)) * (2 as size_t)));
}

fn get_string(cx : context, jsstr : string) -> str unsafe {
    let bytes = get_string_bytes(cx, jsstr);

    // Make a sizing call.
    let len = 0 as size_t;
    if js::JS_EncodeCharacters(*cx, vec::unsafe::to_ptr(bytes),
                               (vec::len(bytes) / 2u) as size_t, ptr::null(),
                               ptr::addr_of(len)) == 0 as JSBool {
        fail;
    }

    let buf = vec::from_elem(len + 1u, 0 as libc::c_char);
    if js::JS_EncodeCharacters(*cx, vec::unsafe::to_ptr(bytes),
                               (vec::len(bytes) / 2u) as size_t,
                               vec::unsafe::to_ptr(buf),
                               ptr::addr_of(len)) == 0 as JSBool {
        fail;
    }

    ret vec::as_buf(buf) {|buf| str::unsafe::from_c_str_len(buf, len as uint) };
}

fn get_int(cx : context, num : jsval) -> i32 unsafe {
    let oparam : i32 = 0i32;
    js::JS_ValueToInt32(*cx, num, ptr::addr_of(oparam));
    ret oparam;
}
