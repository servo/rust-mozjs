#[doc = "

Handy functions for creating class objects and so forth.

"];

use glue::bindgen::GetJSClassHookStubPointer;
use glue::{PROPERTY_STUB, STRICT_PROPERTY_STUB, ENUMERATE_STUB,
              RESOLVE_STUB, CONVERT_STUB};
use libc::c_uint;

pub fn basic_class(np: NamePool, name: ~str) -> JSClass {
    {name: np.add(move name),
     flags: JSCLASS_IS_GLOBAL | JSCLASS_HAS_RESERVED_SLOTS(JSCLASS_GLOBAL_SLOT_COUNT),
     addProperty: GetJSClassHookStubPointer(PROPERTY_STUB) as *u8,
     delProperty: GetJSClassHookStubPointer(PROPERTY_STUB) as *u8,
     getProperty: GetJSClassHookStubPointer(PROPERTY_STUB) as *u8,
     setProperty: GetJSClassHookStubPointer(STRICT_PROPERTY_STUB) as *u8,
     enumerate: GetJSClassHookStubPointer(ENUMERATE_STUB) as *u8,
     resolve: GetJSClassHookStubPointer(RESOLVE_STUB) as *u8,
     convert: GetJSClassHookStubPointer(CONVERT_STUB) as *u8,
     finalize: null(),
     checkAccess: null(),
     call: null(),
     hasInstance: null(),
     construct: null(),
     trace: null(),
     reserved: (null(), null(), null(), null(), null(),  // 05
                null(), null(), null(), null(), null(),  // 10
                null(), null(), null(), null(), null(),  // 15
                null(), null(), null(), null(), null(),  // 20
                null(), null(), null(), null(), null(),  // 25
                null(), null(), null(), null(), null(),  // 30
                null(), null(), null(), null(), null(),  // 35
                null(), null(), null(), null(), null())} // 40
}

pub fn global_class(np: NamePool) -> JSClass {
    basic_class(np, ~"global")
}

pub unsafe fn jsval_to_rust_str(cx: *JSContext, vp: *jsapi::JSString) -> ~str {
  let bytes = JS_EncodeString(cx, vp);
  let s = str::raw::from_c_str(bytes);
  JS_free(cx, cast::reinterpret_cast(&bytes));
  move s
}

pub extern fn debug(cx: *JSContext, argc: c_uint, vp: *JSVal) -> JSBool {
    use io::WriterUtil;

    unsafe {
        let argv = JS_ARGV(cx, vp);
        for uint::range(0u, argc as uint) |i| {
            let jsstr = JS_ValueToString(cx, *ptr::offset(argv, i));
            debug!("%s", jsval_to_rust_str(cx, jsstr));
        }
        JS_SET_RVAL(cx, vp, JSVAL_NULL);
        return 1_i32;
    }
}

pub fn debug_fns(np: NamePool) -> ~[JSFunctionSpec] {
    ~[{name: np.add(~"debug"),
       call: {op: debug,
              info: null()},
       nargs: 0_u16,
       flags: 0_u16,
       selfHostedName: null()}]
}
