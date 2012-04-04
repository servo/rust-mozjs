import libc::*;
import jsapi::*;
import bindgen::*;

#[test]
fn test() {
    let rt = JS_NewRuntime((8 * 1024 * 1024) as uint32_t);
    if ptr::is_null(rt) { fail }
    let cx = JS_NewContext(rt, 8192 as size_t);
    if ptr::is_null(cx) { fail }

    //JS_SetOptions(cx, JSOPTION_VAROBJFIX | JSOPTION_JIT | JSOPTION_METHODJIT);
}
