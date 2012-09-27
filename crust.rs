use jsapi::*;

pub extern fn JS_PropertyStub(++cx: *JSContext, ++obj: JSHandleObject, ++id: JSHandleId, ++vp: JSMutableHandleValue) -> JSBool {
    bindgen::JS_PropertyStub(cx, obj, id, vp)
}

pub extern fn JS_StrictPropertyStub(++cx: *JSContext, ++obj: JSHandleObject, ++id: JSHandleId, ++strict: JSBool, ++vp: JSMutableHandleValue) -> JSBool {
    bindgen::JS_StrictPropertyStub(cx, obj, id, strict, vp)
}

pub extern fn JS_EnumerateStub(++cx: *JSContext, ++obj: JSHandleObject) -> JSBool {
    bindgen::JS_EnumerateStub(cx, obj)
}

pub extern fn JS_ResolveStub(++cx: *JSContext, ++obj: JSHandleObject, ++id: JSHandleId) -> JSBool {
    bindgen::JS_ResolveStub(cx, obj, id)
}

pub extern fn JS_ConvertStub(++cx: *JSContext, ++obj: JSHandleObject, ++_type: JSType, ++vp: JSMutableHandleValue) -> JSBool {
    bindgen::JS_ConvertStub(cx, obj, _type, vp)
}

