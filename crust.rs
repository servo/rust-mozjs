import jsapi::*;

extern fn JS_PropertyStub(++arg0: *JSContext, ++arg1: *JSObject, ++arg2: jsid, ++arg3: *jsval) -> JSBool {
    bindgen::JS_PropertyStub(arg0, arg1, arg2, arg3)
}

extern fn JS_StrictPropertyStub(++arg0: *JSContext, ++arg1: *JSObject, ++arg2: jsid, ++arg3: JSBool, ++arg4: *jsval) -> JSBool {
    bindgen::JS_StrictPropertyStub(arg0, arg1, arg2, arg3, arg4)
}

extern fn JS_EnumerateStub(++arg0: *JSContext, ++arg1: *JSObject) -> JSBool {
    bindgen::JS_EnumerateStub(arg0, arg1)
}

extern fn JS_ResolveStub(++arg0: *JSContext, ++arg1: *JSObject, ++arg2: jsid) -> JSBool {
    bindgen::JS_ResolveStub(arg0, arg1, arg2)
}

extern fn JS_ConvertStub(++arg0: *JSContext, ++arg1: *JSObject, ++arg2: JSType, ++arg3: *jsval) -> JSBool {
    bindgen::JS_ConvertStub(arg0, arg1, arg2, arg3)
}

extern fn JS_FinalizeStub(++_fop: *JSFreeOp, ++_obj: *JSObject) {
    // There doesn't seem to be a native implementation of this anymore?
}

