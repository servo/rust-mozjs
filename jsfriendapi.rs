use jsapi::{JSContext, JSObject, JSNative, JSErrorFormatString, JSClass};

//pub type JSJitPropertyOp = *fn(cx: *JSContext, thisObj: *JSObject, specializedThis: *libc::c_void, vp: *JSVal);
pub type JSJitPropertyOp = *u8;

pub struct JSJitInfo {
    op: JSJitPropertyOp,
    protoID: u32,
    depth: u32,
    isInfallible: bool,
    isConstant: bool
}

//pub type JSJitInfo = JSJitInfo_struct;

#[nolink]
pub extern mod bindgen {
pub fn DefineFunctionWithReserved(cx: *JSContext, obj: *JSObject,
                                  name: *libc::c_char, call: JSNative, nargs: libc::c_uint,
                                  attrs: libc::c_uint) -> *JSObject;
pub fn GetObjectJSClass(obj: *JSObject) -> *JSClass;
pub fn js_GetErrorMessage(userRef: *libc::c_void, locale: *libc::c_char,
                          errorNumber: libc::c_uint) -> *JSErrorFormatString;
pub fn JS_NewObjectWithUniqueType(cx: *JSContext, clasp: *JSClass,
                                  proto: *JSObject, parent: *JSObject) -> *JSObject;
}