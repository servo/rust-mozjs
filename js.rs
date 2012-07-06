import ptr::{null, addr_of};
import result::{result, ok, err};
import libc::c_char;
import name_pool::{name_pool, methods};
import str::unsafe::from_c_str;
import io::writer_util;
import jsapi::{JSBool, JSClass, JSContext, JSErrorReport, JSFunctionSpec,
               JSObject, JSRuntime, JSString, JSVERSION_LATEST, jsuint, jsval,
               uintN};
import jsapi::bindgen::{JS_free, JS_AddObjectRoot, JS_DefineFunctions,
                        JS_DestroyContext, JS_EncodeString, JS_EvaluateScript,
                        JS_Finish, JS_GetContextPrivate, JS_GetPrivate,
                        JS_Init, JS_InitStandardClasses,
                        JS_NewCompartmentAndGlobalObject, JS_NewContext,
                        JS_RemoveObjectRoot, JS_SetContextPrivate,
                        JS_SetErrorReporter, JS_SetOptions, JS_SetPrivate,
                        JS_SetVersion, JS_ValueToString};
import libc::types::common::c99::{int8_t, int16_t, int32_t, int64_t, uint8_t,
                                  uint16_t, uint32_t, uint64_t};

export JSOPTION_STRICT;
export JSOPTION_WERROR;
export JSOPTION_VAROBJFIX;
export JSOPTION_METHODJIT;

export JSCLASS_GLOBAL_FLAGS;

export crust;
export rust;

export jsapi;
export global;

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

export JS_ARGV;
export JS_SET_RVAL;
export JSVAL_VOID;
export JSVAL_NULL;
export JSVAL_ZERO;
export JSVAL_ONE;
export JSVAL_FALSE;
export JSVAL_TRUE;

/* Look in this directory for spidermonkey */
#[link_args = "-L."]
/* Link to the static js library */
#[link_args = "-ljs_static"]
#[link_args = "-lstdc++"]
extern mod m { }

// FIXME: Add the remaining options
const JSOPTION_STRICT: uint32_t =    0b00000000000001u32;
const JSOPTION_WERROR: uint32_t =    0b00000000000010u32;
const JSOPTION_VAROBJFIX: uint32_t = 0b00000000000100u32;
const JSOPTION_METHODJIT: uint32_t = 0b10000000000000u32;

const JSCLASS_GLOBAL_FLAGS: uint32_t = 0x47d00du32;

const default_heapsize: u32 = 8_u32 * 1024_u32 * 1024_u32;
const default_stacksize: uint = 8192u;
const ERR: JSBool = 0_i32;

const JSVAL_VOID: u64 =  0x0001fff2_00000000_u64;
const JSVAL_NULL: u64 =  0x0001fff6_00000000_u64;
const JSVAL_ZERO: u64 =  0x0001fff1_00000000_u64;
const JSVAL_ONE: u64 =   0x0001fff1_00000001_u64;
const JSVAL_FALSE: u64 = 0x0001fff3_00000000_u64;
const JSVAL_TRUE: u64 =  0x0001fff3_00000001_u64;

fn result(n: JSBool) -> result<(),()> {
    if n != ERR {ok(())} else {err(())}
}

type named_functions = @{
    names: [str],
    funcs: [JSFunctionSpec]
};

impl ptr_methods<T: copy> for *T {
    unsafe fn +(idx: uint) -> *T {
        ptr::offset(self, idx)
    }
    unsafe fn [](idx: uint) -> T {
        *(self + idx)
    }
}

unsafe fn JS_ARGV(_cx: *JSContext, vp: *jsval) -> *jsval {
    vp + 2u
}

unsafe fn JS_SET_RVAL(_cx: *JSContext, vp: *jsval, v: jsval) {
    let vp: *mut jsval = unsafe::reinterpret_cast(vp);
    *vp = v;
}

