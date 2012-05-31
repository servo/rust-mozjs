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

crust fn JS_FinalizeStub(++_arg0: *JSContext, ++_arg2: *JSObject) {
    // There doesn't seem to be a native implementation of this anymore?
}

