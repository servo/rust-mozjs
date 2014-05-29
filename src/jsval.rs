/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/. */

use jsapi::{JSObject, JSString, Struct_Unnamed1, JSGCTraceKind, JSTRACE_OBJECT, JSTRACE_STRING};

use std::mem;
use libc::{c_void, uint64_t, c_double, size_t, uintptr_t};

#[cfg(target_word_size = "64")]
static JSVAL_TAG_SHIFT: uint = 47u;

#[repr(u8)]
pub enum ValueType {
    JSVAL_TYPE_DOUBLE              = 0x00,
    JSVAL_TYPE_INT32               = 0x01,
    JSVAL_TYPE_UNDEFINED           = 0x02,
    JSVAL_TYPE_BOOLEAN             = 0x03,
    JSVAL_TYPE_MAGIC               = 0x04,
    JSVAL_TYPE_STRING              = 0x05,
    JSVAL_TYPE_NULL                = 0x06,
    JSVAL_TYPE_OBJECT              = 0x07,

    /* These never appear in a jsval; they are only provided as an out-of-band value. */
    JSVAL_TYPE_UNKNOWN             = 0x20,
    JSVAL_TYPE_MISSING             = 0x21
}

#[cfg(target_word_size = "64")]
static JSVAL_TAG_MAX_DOUBLE: u32 = 0x1FFF0u32;

#[cfg(target_word_size = "32")]
static JSVAL_TAG_CLEAR: u32 = 0xFFFFFF80;

#[cfg(target_word_size = "64")]
#[repr(u32)]
enum ValueTag {
    JSVAL_TAG_DOUBLE               = JSVAL_TAG_MAX_DOUBLE | (JSVAL_TYPE_DOUBLE as u32),
    JSVAL_TAG_INT32                = JSVAL_TAG_MAX_DOUBLE | (JSVAL_TYPE_INT32 as u32),
    JSVAL_TAG_UNDEFINED            = JSVAL_TAG_MAX_DOUBLE | (JSVAL_TYPE_UNDEFINED as u32),
    JSVAL_TAG_STRING               = JSVAL_TAG_MAX_DOUBLE | (JSVAL_TYPE_STRING as u32),
    JSVAL_TAG_BOOLEAN              = JSVAL_TAG_MAX_DOUBLE | (JSVAL_TYPE_BOOLEAN as u32),
    JSVAL_TAG_MAGIC                = JSVAL_TAG_MAX_DOUBLE | (JSVAL_TYPE_MAGIC as u32),
    JSVAL_TAG_NULL                 = JSVAL_TAG_MAX_DOUBLE | (JSVAL_TYPE_NULL as u32),
    JSVAL_TAG_OBJECT               = JSVAL_TAG_MAX_DOUBLE | (JSVAL_TYPE_OBJECT as u32),
}

#[cfg(target_word_size = "32")]
#[repr(u32)]
enum ValueTag {
    JSVAL_TAG_PRIVATE              = 0,
    JSVAL_TAG_INT32                = JSVAL_TAG_CLEAR as u32 | (JSVAL_TYPE_INT32 as u32),
    JSVAL_TAG_UNDEFINED            = JSVAL_TAG_CLEAR as u32 | (JSVAL_TYPE_UNDEFINED as u32),
    JSVAL_TAG_STRING               = JSVAL_TAG_CLEAR as u32 | (JSVAL_TYPE_STRING as u32),
    JSVAL_TAG_BOOLEAN              = JSVAL_TAG_CLEAR as u32 | (JSVAL_TYPE_BOOLEAN as u32),
    JSVAL_TAG_MAGIC                = JSVAL_TAG_CLEAR as u32 | (JSVAL_TYPE_MAGIC as u32),
    JSVAL_TAG_NULL                 = JSVAL_TAG_CLEAR as u32 | (JSVAL_TYPE_NULL as u32),
    JSVAL_TAG_OBJECT               = JSVAL_TAG_CLEAR as u32 | (JSVAL_TYPE_OBJECT as u32),
}

#[cfg(target_word_size = "64")]
#[repr(u64)]
enum ValueShiftedTag {
    JSVAL_SHIFTED_TAG_MAX_DOUBLE   = (((JSVAL_TAG_MAX_DOUBLE as u64) << JSVAL_TAG_SHIFT) | 0xFFFFFFFFu64),
    JSVAL_SHIFTED_TAG_INT32        = ((JSVAL_TAG_INT32 as u64)      << JSVAL_TAG_SHIFT),
    JSVAL_SHIFTED_TAG_UNDEFINED    = ((JSVAL_TAG_UNDEFINED as u64)  << JSVAL_TAG_SHIFT),
    JSVAL_SHIFTED_TAG_STRING       = ((JSVAL_TAG_STRING as u64)     << JSVAL_TAG_SHIFT),
    JSVAL_SHIFTED_TAG_BOOLEAN      = ((JSVAL_TAG_BOOLEAN as u64)    << JSVAL_TAG_SHIFT),
    JSVAL_SHIFTED_TAG_MAGIC        = ((JSVAL_TAG_MAGIC as u64)      << JSVAL_TAG_SHIFT),
    JSVAL_SHIFTED_TAG_NULL         = ((JSVAL_TAG_NULL as u64)       << JSVAL_TAG_SHIFT),
    JSVAL_SHIFTED_TAG_OBJECT       = ((JSVAL_TAG_OBJECT as u64)     << JSVAL_TAG_SHIFT)
}


#[cfg(target_word_size = "64")]
static JSVAL_PAYLOAD_MASK: u64 = 0x00007FFFFFFFFFFF;

#[deriving(PartialEq,Clone)]
pub struct JSVal {
    v: u64
}

#[cfg(target_word_size = "64")]
#[inline(always)]
fn BuildJSVal(tag: ValueTag, payload: u64) -> JSVal {
    Union_jsval_layout {
        v: ((tag as u32 as u64) << JSVAL_TAG_SHIFT) | payload
    }
}

#[cfg(target_word_size = "32")]
#[inline(always)]
fn BuildJSVal(tag: ValueTag, payload: u64) -> JSVal {
    JSVal {
        v: ((tag as u32 as u64) << 32) | payload
    }
}

#[inline(always)]
pub fn NullValue() -> JSVal {
    BuildJSVal(JSVAL_TAG_NULL, 0)
}

#[inline(always)]
pub fn UndefinedValue() -> JSVal {
    BuildJSVal(JSVAL_TAG_UNDEFINED, 0)
}

#[inline(always)]
pub fn Int32Value(i: i32) -> JSVal {
    BuildJSVal(JSVAL_TAG_INT32, i as u32 as u64)
}

#[cfg(target_word_size = "64")]
#[inline(always)]
pub fn DoubleValue(f: f64) -> JSVal {
    let bits: u64 = unsafe { mem::transmute(f) };
    assert!(bits <= JSVAL_SHIFTED_TAG_MAX_DOUBLE as u64)
    Union_jsval_layout {
        v: bits
    }
}

#[cfg(target_word_size = "32")]
#[inline(always)]
pub fn DoubleValue(f: f64) -> JSVal {
    let bits: u64 = unsafe { mem::transmute(f) };
    let val = JSVal {
        v: bits
    };
    assert!(val.is_double());
    val
}

#[inline(always)]
pub fn UInt32Value(ui: u32) -> JSVal {
    if ui > 0x7fffffff {
        DoubleValue(ui as f64)
    } else {
        Int32Value(ui as i32)
    }
}

#[cfg(target_word_size = "64")]
#[inline(always)]
pub fn StringValue(s: &JSString) -> JSVal {
    let bits = s as *const JSString as uint as u64;
    assert!((bits >> JSVAL_TAG_SHIFT) == 0);
    BuildJSVal(JSVAL_TAG_STRING, bits)
}

#[cfg(target_word_size = "32")]
#[inline(always)]
pub fn StringValue(s: &JSString) -> JSVal {
    let bits = s as *const JSString as uint as u64;
    BuildJSVal(JSVAL_TAG_STRING, bits)
}

#[inline(always)]
pub fn BooleanValue(b: bool) -> JSVal {
    BuildJSVal(JSVAL_TAG_BOOLEAN, b as u64)
}

#[cfg(target_word_size = "64")]
#[inline(always)]
pub fn ObjectValue(o: &JSObject) -> JSVal {
    let bits = o as *const JSObject as uint as u64;
    assert!((bits >> JSVAL_TAG_SHIFT) == 0);
    BuildJSVal(JSVAL_TAG_OBJECT, bits)
}

#[cfg(target_word_size = "32")]
#[inline(always)]
pub fn ObjectValue(o: &JSObject) -> JSVal {
    let bits = o as *const JSObject as uint as u64;
    BuildJSVal(JSVAL_TAG_OBJECT, bits)
}

#[inline(always)]
pub fn ObjectOrNullValue(o: *mut JSObject) -> JSVal {
    if o.is_null() {
        NullValue()
    } else {
        ObjectValue(unsafe { &*o })
    }
}

#[cfg(target_word_size = "64")]
#[inline(always)]
pub fn PrivateValue(o: *const c_void) -> JSVal {
    let ptrBits = o as uint as u64;
    assert!((ptrBits & 1) == 0);
    Union_jsval_layout {
        v: ptrBits >> 1
    }
}

#[cfg(target_word_size = "32")]
#[inline(always)]
pub fn PrivateValue(o: *const c_void) -> JSVal {
    let ptrBits = o as uint as u64;
    assert!((ptrBits & 1) == 0);
    BuildJSVal(JSVAL_TAG_PRIVATE, ptrBits)
}

impl JSVal {
    #[cfg(target_word_size = "64")]
    pub fn is_undefined(&self) -> bool {
        self.v == JSVAL_SHIFTED_TAG_UNDEFINED as u64
    }

    #[cfg(target_word_size = "32")]
    pub fn is_undefined(&self) -> bool {
        (self.v >> 32) == JSVAL_TAG_UNDEFINED as u64
    }

    #[cfg(target_word_size = "64")]
    pub fn is_null(&self) -> bool {
        self.v == JSVAL_SHIFTED_TAG_NULL as u64
    }

    #[cfg(target_word_size = "32")]
    pub fn is_null(&self) -> bool {
        (self.v >> 32) == JSVAL_TAG_NULL as u64
    }

    pub fn is_null_or_undefined(&self) -> bool {
        self.is_null() || self.is_undefined()
    }

    #[cfg(target_word_size = "64")]
    pub fn is_boolean(&self) -> bool {
        self.v == JSVAL_SHIFTED_TAG_BOOLEAN as u64
    }

    #[cfg(target_word_size = "32")]
    pub fn is_boolean(&self) -> bool {
        (self.v >> 32) == JSVAL_TAG_BOOLEAN as u64
    }

    #[cfg(target_word_size = "64")]
    pub fn is_double(&self) -> bool {
        self.v <= JSVAL_SHIFTED_TAG_MAX_DOUBLE as u64
    }

    #[cfg(target_word_size = "32")]
    pub fn is_double(&self) -> bool {
        (self.v >> 32) <= JSVAL_TAG_CLEAR as u64
    }

    #[cfg(target_word_size = "64")]
    pub fn is_primitive(&self) -> bool {
        static JSVAL_UPPER_EXCL_SHIFTED_TAG_OF_PRIMITIVE_SET: u64 = JSVAL_SHIFTED_TAG_OBJECT as u64;
        self.v < JSVAL_UPPER_EXCL_SHIFTED_TAG_OF_PRIMITIVE_SET
    }

    #[cfg(target_word_size = "32")]
    pub fn is_primitive(&self) -> bool {
        static JSVAL_UPPER_EXCL_TAG_OF_PRIMITIVE_SET: u64 = JSVAL_TAG_OBJECT as u64;
        (self.v >> 32) < JSVAL_UPPER_EXCL_TAG_OF_PRIMITIVE_SET
    }

    #[cfg(target_word_size = "64")]
    pub fn is_string(&self) -> bool {
        (self.v >> JSVAL_TAG_SHIFT) == JSVAL_TAG_STRING as u64
    }

    #[cfg(target_word_size = "32")]
    pub fn is_string(&self) -> bool {
        (self.v >> 32) == JSVAL_TAG_STRING as u64
    }

    #[cfg(target_word_size = "64")]
    pub fn is_object(&self) -> bool {
        assert!((self.v >> JSVAL_TAG_SHIFT) <= JSVAL_TAG_OBJECT as u64);
        self.v >= JSVAL_SHIFTED_TAG_OBJECT as u64
    }

    #[cfg(target_word_size = "32")]
    pub fn is_object(&self) -> bool {
        (self.v >> 32) == JSVAL_TAG_OBJECT as u64
    }

    #[cfg(target_word_size = "64")]
    pub fn to_boolean(&self) -> bool {
        assert!(self.is_boolean());
        (self.v & JSVAL_PAYLOAD_MASK) != 0
    }

    #[cfg(target_word_size = "32")]
    pub fn to_boolean(&self) -> bool {
        (self.v & 0x00000000FFFFFFFF) != 0
    }

    pub fn to_object(&self) -> *mut JSObject {
        assert!(self.is_object());
        self.to_object_or_null()
    }

    #[cfg(target_word_size = "64")]
    pub fn is_object_or_null(&self) -> bool {
        static JSVAL_LOWER_INCL_SHIFTED_TAG_OF_OBJ_OR_NULL_SET: u64 = JSVAL_SHIFTED_TAG_NULL as u64;
        assert!((self.v >> JSVAL_TAG_SHIFT) <= JSVAL_TAG_OBJECT as u64);
        self.v >= JSVAL_LOWER_INCL_SHIFTED_TAG_OF_OBJ_OR_NULL_SET
    }

    #[cfg(target_word_size = "32")]
    pub fn is_object_or_null(&self) -> bool {
        static JSVAL_LOWER_INCL_TAG_OF_OBJ_OR_NULL_SET: u64 = JSVAL_TAG_NULL as u64;
        assert!((self.v >> 32) <= JSVAL_TAG_OBJECT as u64);
        (self.v >> 32) >= JSVAL_LOWER_INCL_TAG_OF_OBJ_OR_NULL_SET
    }

    #[cfg(target_word_size = "64")]
    pub fn to_object_or_null(&self) -> *mut JSObject {
        assert!(self.is_object_or_null());
        let ptrBits = self.v & JSVAL_PAYLOAD_MASK;
        assert!((ptrBits & 0x7) == 0);
        ptrBits as uint as *mut JSObject
    }

    #[cfg(target_word_size = "32")]
    pub fn to_object_or_null(&self) -> *mut JSObject {
        assert!(self.is_object_or_null());
        let ptrBits: u32 = (self.v & 0x00000000FFFFFFFF) as u32;
        ptrBits as *mut JSObject
    }

    #[cfg(target_word_size = "64")]
    pub fn to_private(&self) -> *const c_void {
        assert!(self.is_double());
        assert!((self.v & 0x8000000000000000u64) == 0);
        (self.v << 1) as uint as *const c_void
    }

    #[cfg(target_word_size = "32")]
    pub fn to_private(&self) -> *const c_void {
        let ptrBits: u32 = (self.v & 0x00000000FFFFFFFF) as u32;
        ptrBits as *const c_void
    }

    #[cfg(target_word_size = "64")]
    pub fn is_gcthing(&self) -> bool {
        static JSVAL_LOWER_INCL_SHIFTED_TAG_OF_GCTHING_SET: u64 = JSVAL_SHIFTED_TAG_STRING as u64;
        self.v >= JSVAL_LOWER_INCL_SHIFTED_TAG_OF_GCTHING_SET
    }

    #[cfg(target_word_size = "32")]
    pub fn is_gcthing(&self) -> bool {
        static JSVAL_LOWER_INCL_TAG_OF_GCTHING_SET: u64 = JSVAL_TAG_STRING as u64;
        (self.v >> 32) >= JSVAL_LOWER_INCL_TAG_OF_GCTHING_SET
    }

    #[cfg(target_word_size = "64")]
    pub fn to_gcthing(&self) -> *mut c_void {
        assert!(self.is_gcthing());
        let ptrBits = self.v & JSVAL_PAYLOAD_MASK;
        assert!((ptrBits & 0x7) == 0);
        ptrBits as *mut c_void
    }

    #[cfg(target_word_size = "32")]
    pub fn to_gcthing(&self) -> *mut c_void {
        assert!(self.is_gcthing());
        let ptrBits: u32 = (self.v & 0x00000000FFFFFFFF) as u32;
        ptrBits as *mut c_void
    }

    pub fn is_markable(&self) -> bool {
        self.is_gcthing() && !self.is_null()
    }

    pub fn trace_kind(&self) -> JSGCTraceKind {
        assert!(self.is_markable());
        if self.is_object() {
            JSTRACE_OBJECT
        } else {
            JSTRACE_STRING
        }
    }
}
