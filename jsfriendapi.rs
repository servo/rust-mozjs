mod bindgen {

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
    
}