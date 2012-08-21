#[doc = "

Handy functions for creating class objects and so forth.

"];

// Can't use spidermonkey::crust::* versions due to Rust #2440

import libc::c_uint;
export basic_class;
export global_class;
export debug_fns;
export jsval_to_rust_str;

fn basic_class(np: name_pool, -name: ~str) -> JSClass {
    {name: np.add(name),
     flags: 0x48000_u32,
     addProperty: crust::JS_PropertyStub,
     delProperty: crust::JS_PropertyStub,
     getProperty: crust::JS_PropertyStub,
     setProperty: crust::JS_StrictPropertyStub,
     enumerate: crust::JS_EnumerateStub,
     resolve: crust::JS_ResolveStub,
     convert: crust::JS_ConvertStub,
     finalize: null(),
     checkAccess: null(),
     call: null(),
     construct: null(),
     hasInstance: null(),
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

fn global_class(np: name_pool) -> JSClass {
    basic_class(np, ~"global")
}

unsafe fn jsval_to_rust_str(cx: *JSContext, vp: *jsapi::JSString) -> ~str {
  let bytes = JS_EncodeString(cx, vp);
  let s = str::unsafe::from_c_str(bytes);
  JS_free(cx, unsafe::reinterpret_cast(bytes));
  s
}

extern fn debug(cx: *JSContext, argc: c_uint, vp: *jsval) -> JSBool {
    import io::WriterUtil;

    unsafe {
        let argv = JS_ARGV(cx, vp);
        for uint::range(0u, argc as uint) |i| {
            let jsstr = JS_ValueToString(cx, *ptr::offset(argv, i));
            #debug["%s", jsval_to_rust_str(cx, jsstr)];
        }
        JS_SET_RVAL(cx, vp, JSVAL_NULL);
        return 1_i32;
    }
}

fn debug_fns(np: name_pool) -> ~[JSFunctionSpec] {
    ~[{name: np.add(~"debug"),
       call: debug,
       nargs: 0_u16,
       flags: 0_u16}]
}
