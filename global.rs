#[doc = "

Handy functions for creating class objects and so forth.

"];

import name_pool::add;

// Can't use spidermonkey::crust::* versions due to Rust #2440

export basic_class;
export global_class;
export debug_fns;

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
     reserved0: null(),
     checkAccess: null(),
     call: null(),
     construct: null(),
     xdrObject: null(),
     hasInstance: null(),
     trace: null(),
     reserved1: null(),
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

extern fn debug(cx: *JSContext, argc: uintN, vp: *jsval) -> JSBool {
    import io::writer_util;

    #debug["debug() called with %? arguments", argc];

    unsafe {
        let argv = JS_ARGV(cx, vp);
        for uint::range(0u, argc as uint) |i| {
            let jsstr = JS_ValueToString(cx, argv[i]);
            let bytes = JS_EncodeString(cx, jsstr);
            let str = str::unsafe::from_c_str(bytes);
            JS_free(cx, unsafe::reinterpret_cast(bytes));
            #debug["%s", str];
        }
        JS_SET_RVAL(cx, vp, JSVAL_NULL);
        ret 1_i32;
    }
}

fn debug_fns(np: name_pool) -> ~[JSFunctionSpec] {
    ~[{name: np.add(~"debug"),
       call: debug,
       nargs: 0_u16,
       flags: 0_u16}]
}
