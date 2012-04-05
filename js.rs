import libc::types::common::c99::*;

export jsapi;

// These are just macros in jsapi.h
import JS_NewRuntime = jsapi::bindgen::JS_Init;
export JS_NewRuntime;
import JS_DestroyRuntime = jsapi::bindgen::JS_Finish;
export JS_DestroyRuntime;
/*
FIXME: Not sure where JS_Lock is
import JS_LockRuntime = jsapi::bindgen::JS_Lock;
export JS_LockRuntime;
import JS_UnlockRuntime = jsapi::bindgen::JS_Unlock;
export JS_UnlockRuntime;
*/

#[link_args = "-L."]
native mod m { }

// FIXME: Add the remaining options
const JSOPTION_STRICT: uint32_t =    0b00000000000001u32;
const JSOPTION_WERROR: uint32_t =    0b00000000000010u32;
const JSOPTION_VAROBJFIX: uint32_t = 0b00000000000100u32;
const JSOPTION_METHODJIT: uint32_t = 0b10000000000000u32;

const JSCLASS_GLOBAL_FLAGS: uint32_t = 0x47d00du32;

mod crust {
    import jsapi::*;

    crust fn JS_PropertyStub(++arg0: *JSContext, ++arg1: *JSObject, ++arg2: jsid, ++arg3: *jsval) -> JSBool {
        bindgen::JS_PropertyStub(arg0, arg1, arg2, arg3)
    }

    crust fn JS_StrictPropertyStub(++arg0: *JSContext, ++arg1: *JSObject, ++arg2: jsid, ++arg3: JSBool, ++arg4: *jsval) -> JSBool {
        bindgen::JS_StrictPropertyStub(arg0, arg1, arg2, arg3, arg4)
    }

    crust fn JS_EnumerateStub(++arg0: *JSContext, ++arg1: *JSObject) -> JSBool {
        bindgen::JS_EnumerateStub(arg0, arg1)
    }

    crust fn JS_ResolveStub(++arg0: *JSContext, ++arg1: *JSObject, ++arg2: jsid) -> JSBool {
        bindgen::JS_ResolveStub(arg0, arg1, arg2)
    }

    crust fn JS_ConvertStub(++arg0: *JSContext, ++arg1: *JSObject, ++arg2: JSType, ++arg3: *jsval) -> JSBool {
        bindgen::JS_ConvertStub(arg0, arg1, arg2, arg3)
    }

    crust fn JS_FinalizeStub(++arg0: *JSContext, ++arg2: *JSObject) {
        // FIXME: Can't find the native implementation of this
    }
}
