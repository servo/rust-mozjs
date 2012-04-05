import libc::*;
import jsapi::*;
import bindgen::*;

#[test]
fn test() {
    // From the JSAPI user's guide
    crust fn report_error(_cx: *JSContext, message: *c_char, report: *JSErrorReport) unsafe {
        let filename = if ptr::is_not_null((*report).filename) {
            str::unsafe::from_c_str((*report).filename)
        } else {
            "<no filename>"
        };

        #error("%s:%u:%s", filename, (*report).lineno as uint, str::unsafe::from_c_str(message));
    }


    let rt = JS_NewRuntime((8 * 1024 * 1024) as uint32_t);
    if ptr::is_null(rt) { fail }
    let cx = JS_NewContext(rt, 8192 as size_t);
    if ptr::is_null(cx) { fail }

    JS_SetOptions(cx, JSOPTION_VAROBJFIX | JSOPTION_METHODJIT);
    JS_SetVersion(cx, JSVERSION_LATEST);
    JS_SetErrorReporter(cx, report_error);

    let global_name = "global";
    let global_class = {
        name: str::as_c_str(global_name) {|buf| buf},
        flags: JSCLASS_GLOBAL_FLAGS,
        addProperty: crust::JS_PropertyStub,
        delProperty: crust::JS_PropertyStub,
        getProperty: crust::JS_PropertyStub,
        setProperty: crust::JS_StrictPropertyStub,
        enumerate: crust::JS_EnumerateStub,
        resolve: crust::JS_ResolveStub,
        convert: crust::JS_ConvertStub,
        finalize: crust::JS_FinalizeStub,
        checkAccess: ptr::null(),
        call: ptr::null(),
        construct: ptr::null(),
        hasInstance: ptr::null(),
        trace: ptr::null(),
        reserved: (
            ptr::null(), ptr::null(), ptr::null(), ptr::null(),
            ptr::null(), ptr::null(), ptr::null(), ptr::null(),
            ptr::null(), ptr::null(), ptr::null(), ptr::null(),
            ptr::null(), ptr::null(), ptr::null(), ptr::null(),
            ptr::null(), ptr::null(), ptr::null(), ptr::null(),
            ptr::null(), ptr::null(), ptr::null(), ptr::null(),
            ptr::null(), ptr::null(), ptr::null(), ptr::null(),
            ptr::null(), ptr::null(), ptr::null(), ptr::null(),
            ptr::null(), ptr::null(), ptr::null(), ptr::null(),
            ptr::null(), ptr::null(), ptr::null(), ptr::null()
        )
    };

    /* Create the global object in a new compartment. */
    let global = JS_NewCompartmentAndGlobalObject(cx, ptr::addr_of(global_class), ptr::null());
    if ptr::is_null(global) { fail }

    /* Populate the global object with the standard globals, like Object and Array. */
    if !(JS_InitStandardClasses(cx, global) as bool) { fail }

    let code = "print(\"this is a test\")";
    let script = str::as_c_str(code) {|codebuf|
        str::as_c_str("test") {|name|
            JS_CompileScript(cx, global, codebuf, str::len(code), name, 0 as c_uint)
        }
    };
    JS_ExecuteScript(cx, global, script, ptr::null());

    JS_DestroyContext(cx);
    JS_DestroyRuntime(rt);
    JS_ShutDown();
}
