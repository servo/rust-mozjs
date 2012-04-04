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

    /*
    let global_name = "global";
    let global_class = {
        name: str::as_c_str(global_name) {|buf| buf},
        flags:
    };

    let global = JS_NewCompartmentAndGlobalObject(cx, global_class, ptr::null());
    if ptr::is_null(global) { fail }
    */
}
