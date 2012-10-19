use ptr::{null, addr_of};
use result::{Result, Ok, Err};
use libc::{c_char, c_uint};
use name_pool::{NamePool, add};
use str::raw::from_c_str;
use io::WriterUtil;
use jsapi::{JSBool, JSClass, JSContext, JSErrorReport, JSFunctionSpec,
               JSObject, JSRuntime, JSString, JSVERSION_LATEST, JSVal, jsid,
               JSPropertySpec, JSPropertyOp, JSStrictPropertyOp, JSProto_LIMIT};
use jsapi::bindgen::{JS_free, JS_AddObjectRoot, JS_DefineFunctions,
                        JS_DestroyContext, JS_EncodeString, JS_EvaluateScript,
                        JS_Finish, JS_GetContextPrivate, JS_GetPrivate,
                        JS_Init, JS_InitStandardClasses,
                        JS_NewGlobalObject, JS_NewContext,
                        JS_RemoveObjectRoot, JS_SetContextPrivate,
                        JS_SetErrorReporter, JS_SetOptions, JS_SetPrivate,
                        JS_SetVersion, JS_ValueToString, JS_DefineProperties,
                        JS_DefineProperty, JS_NewObject, JS_ComputeThis};
use libc::types::common::c99::{int8_t, int16_t, int32_t, int64_t, uint8_t,
                                  uint16_t, uint32_t, uint64_t};
use jsval::{JSVAL_TO_OBJECT, JSVAL_IS_PRIMITIVE};
use glue::bindgen::{RUST_JSVAL_TO_OBJECT, RUST_JSVAL_IS_PRIMITIVE};
use rust::jsobj;
pub use rust;

pub use NamePool;
pub use mod jsapi;
pub use mod glue;
pub use mod crust;

// These are just macros in jsapi.h
pub use JS_NewRuntime = jsapi::bindgen::JS_Init;
pub use JS_DestroyRuntime = jsapi::bindgen::JS_Finish;
/*
FIXME: Not sure where JS_Lock is
pub use JS_LockRuntime = jsapi::bindgen::JS_Lock;
pub use JS_UnlockRuntime = jsapi::bindgen::JS_Unlock;
*/

// FIXME: Add the remaining options
pub const JSOPTION_STRICT: uint32_t =    0b00000000000001u32;
pub const JSOPTION_WERROR: uint32_t =    0b00000000000010u32;
pub const JSOPTION_VAROBJFIX: uint32_t = 0b00000000000100u32;
pub const JSOPTION_METHODJIT: uint32_t = (1 << 14) as u32;
pub const JSOPTION_TYPE_INFERENCE: uint32_t = (1 << 18) as u32;

pub const default_heapsize: u32 = 8_u32 * 1024_u32 * 1024_u32;
pub const default_stacksize: uint = 8192u;
pub const ERR: JSBool = 0_i32;

pub const JSVAL_TAG_MAX_DOUBLE: u64 = 0x1FFF0;

pub const JSVAL_TYPE_DOUBLE: u64 = 0x00;
pub const JSVAL_TYPE_INT32: u64 = 0x01;
pub const JSVAL_TYPE_UNDEFINED: u64 = 0x02;
pub const JSVAL_TYPE_BOOLEAN: u64 = 0x03;
pub const JSVAL_TYPE_MAGIC: u64 = 0x04;
pub const JSVAL_TYPE_STRING: u64 = 0x05;
pub const JSVAL_TYPE_NULL: u64 = 0x06;
pub const JSVAL_TYPE_OBJECT: u64 = 0x07;
pub const JSVAL_TYPE_UNKNOWN: u64 = 0x20;

pub const JSVAL_TAG_SHIFT: int = 47;

//The following constants are totally broken on non-64bit platforms.
//See jsapi.h for the proper macro definitions.
pub const JSVAL_VOID: u64 = (JSVAL_TAG_MAX_DOUBLE | JSVAL_TYPE_UNKNOWN) << JSVAL_TAG_SHIFT;
pub const JSVAL_NULL: u64 = (JSVAL_TAG_MAX_DOUBLE | JSVAL_TYPE_NULL) << JSVAL_TAG_SHIFT;
pub const JSVAL_ZERO: u64 = (JSVAL_TAG_MAX_DOUBLE | JSVAL_TYPE_INT32) << JSVAL_TAG_SHIFT;
pub const JSVAL_ONE: u64 = ((JSVAL_TAG_MAX_DOUBLE | JSVAL_TYPE_INT32) << JSVAL_TAG_SHIFT) | 1;
pub const JSVAL_FALSE: u64 = (JSVAL_TAG_MAX_DOUBLE | JSVAL_TYPE_BOOLEAN) << JSVAL_TAG_SHIFT;
pub const JSVAL_TRUE: u64 = ((JSVAL_TAG_MAX_DOUBLE | JSVAL_TYPE_BOOLEAN) << JSVAL_TAG_SHIFT) | 1;

pub const JSPROP_ENUMERATE: c_uint = 0x01;
pub const JSPROP_READONLY: c_uint  = 0x02;
pub const JSPROP_SHARED: c_uint =    0x40;
pub const JSPROP_NATIVE_ACCESSORS: c_uint = 0x08;

pub const JSCLASS_RESERVED_SLOTS_SHIFT: c_uint = 8;
pub const JSCLASS_RESERVED_SLOTS_WIDTH: c_uint = 8;
pub const JSCLASS_RESERVED_SLOTS_MASK: c_uint = ((1 << JSCLASS_RESERVED_SLOTS_WIDTH) - 1);

pub const JSCLASS_HIGH_FLAGS_SHIFT: c_uint =
    JSCLASS_RESERVED_SLOTS_SHIFT + JSCLASS_RESERVED_SLOTS_WIDTH;
pub const JSCLASS_IS_GLOBAL: c_uint = (1<<(JSCLASS_HIGH_FLAGS_SHIFT+1));

pub const JSCLASS_GLOBAL_SLOT_COUNT: c_uint = JSProto_LIMIT * 3 + 24;

pub pure fn JSCLASS_HAS_RESERVED_SLOTS(n: c_uint) -> c_uint {
    (n & JSCLASS_RESERVED_SLOTS_MASK) << JSCLASS_RESERVED_SLOTS_SHIFT
}

pub fn result(n: JSBool) -> Result<(),()> {
    if n != ERR {Ok(())} else {Err(())}
}
pub fn result_obj(o: jsobj) -> Result<jsobj, ()> {
    if o.ptr != null() {Ok(o)} else {Err(())}
}

pub type named_functions = @{
    names: ~[~str],
    funcs: ~[JSFunctionSpec]
};

#[inline(always)]
pub unsafe fn JS_ARGV(_cx: *JSContext, vp: *JSVal) -> *JSVal {
    ptr::offset(vp, 2u)
}

pub unsafe fn JS_SET_RVAL(_cx: *JSContext, vp: *JSVal, v: JSVal) {
    let vp: *mut JSVal = cast::reinterpret_cast(&vp);
    *vp = v;
}

#[inline(always)]
pub unsafe fn JS_THIS_OBJECT(cx: *JSContext, vp: *JSVal) -> *JSObject unsafe {
    let r = JSVAL_TO_OBJECT(
        if JSVAL_IS_PRIMITIVE(*ptr::offset(vp, 1)) {
            JS_ComputeThis(cx, vp)
        } else {
            *ptr::offset(vp, 1)
        });
    r
}

// This is a duplication of the shadow stuff from jsfriendapi.h.  Here
// there be dragons!
mod shadow {
    struct TypeObject {
        proto: *JSObject
    }

    struct BaseShape {
        clasp: *JSClass,
        parent: *JSObject
    }

    struct Shape {
        base: *BaseShape,
        _1: jsid,
        slotInfo: u32
    }
    const FIXED_SLOTS_SHIFT: u32 = 27;

    pub struct Object {
        shape: *Shape,
        objType: *TypeObject,
        slots: *JSVal,
        _1: *JSVal,
    }

    impl Object {
        #[inline(always)]
        pure fn numFixedSlots() -> libc::size_t unsafe {
            ((*self.shape).slotInfo >> FIXED_SLOTS_SHIFT) as libc::size_t
        }
        
        #[inline(always)]
        fn fixedSlots() -> *jsval {
            (ptr::offset(ptr::to_unsafe_ptr(&self), 1)) as *JSVal
        }

        // Like slotRef, but just returns the value, not a reference
        #[inline(always)]
        pure fn slotVal(slot: libc::size_t) -> JSVal unsafe {
            let nfixed : libc::size_t = self.numFixedSlots();
            if slot < nfixed {
                return *ptr::offset(self.fixedSlots(), slot as uint)
            }
            return *ptr::offset(self.slots, (slot - nfixed) as uint)
        }
    }
}

#[inline(always)]
pub unsafe fn GetReservedSlot(obj: *JSObject, slot: libc::size_t) -> JSVal {
    let s = obj as *shadow::Object;
    return (*s).slotVal(slot)
}
