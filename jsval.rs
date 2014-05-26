/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/. */

use jsapi::{JSObject, JSString, JSGCTraceKind, JSTRACE_OBJECT, JSTRACE_STRING};

use libc::c_void;
use std::cast;

static JSVAL_TAG_SHIFT: int = 47;

#[repr(u8)]
enum ValueType {
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

static JSVAL_TAG_MAX_DOUBLE: u32 = 0x1FFF0u32;

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


static JSVAL_PAYLOAD_MASK: u64 = 0x00007FFFFFFFFFFF;

// JSVal was originally type of u64.
// now this become {u64} because of the union abi issue on ARM arch. See #398.
#[deriving(Eq,Clone)]
pub struct JSVal {
    pub v: u64
}

#[inline(always)]
fn BuildJSVal(tag: ValueTag, payload: u64) -> JSVal {
    JSVal {
        v: ((tag as u32 as u64) << JSVAL_TAG_SHIFT) | payload
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

#[inline(always)]
pub fn DoubleValue(f: f64) -> JSVal {
    let bits: u64 = unsafe { cast::transmute(f) };
    assert!(bits <= JSVAL_SHIFTED_TAG_MAX_DOUBLE as u64)
    JSVal {
        v: bits
    }
}

#[inline(always)]
pub fn UInt32Value(ui: u32) -> JSVal {
    if ui > 0x7fffffff {
        DoubleValue(ui as f64)
    } else {
        Int32Value(ui as i32)
    }
}

#[inline(always)]
pub fn StringValue(s: &JSString) -> JSVal {
    let bits = s as *JSString as uint as u64;
    assert!((bits >> JSVAL_TAG_SHIFT) == 0);
    BuildJSVal(JSVAL_TAG_STRING, bits)
}

#[inline(always)]
pub fn BooleanValue(b: bool) -> JSVal {
    BuildJSVal(JSVAL_TAG_BOOLEAN, b as u64)
}

#[inline(always)]
pub fn ObjectValue(o: &JSObject) -> JSVal {
    let bits = o as *JSObject as uint as u64;
    assert!((bits >> JSVAL_TAG_SHIFT) == 0);
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

#[inline(always)]
pub fn PrivateValue(o: *c_void) -> JSVal {
    let ptrBits = o as uint as u64;
    assert!((ptrBits & 1) == 0);
    JSVal {
        v: ptrBits >> 1
    }
}

impl JSVal {
    pub fn is_undefined(&self) -> bool {
        self.v == JSVAL_SHIFTED_TAG_UNDEFINED as u64
    }

    pub fn is_null(&self) -> bool {
        self.v == JSVAL_SHIFTED_TAG_NULL as u64
    }

    pub fn is_null_or_undefined(&self) -> bool {
        self.is_null() || self.is_undefined()
    }

    pub fn is_double(&self) -> bool {
        self.v <= JSVAL_SHIFTED_TAG_MAX_DOUBLE as u64
    }

    pub fn is_primitive(&self) -> bool {
        static JSVAL_UPPER_EXCL_SHIFTED_TAG_OF_PRIMITIVE_SET: u64 = JSVAL_SHIFTED_TAG_OBJECT as u64;
        self.v < JSVAL_UPPER_EXCL_SHIFTED_TAG_OF_PRIMITIVE_SET
    }

    pub fn is_string(&self) -> bool {
        (self.v >> JSVAL_TAG_SHIFT) == JSVAL_TAG_STRING as u64
    }

    pub fn is_object(&self) -> bool {
        assert!((self.v >> JSVAL_TAG_SHIFT) <= JSVAL_TAG_OBJECT as u64);
        self.v >= JSVAL_SHIFTED_TAG_OBJECT as u64
    }

    pub fn to_object(&self) -> *mut JSObject {
        assert!(self.is_object());
        self.to_object_or_null()
    }

    pub fn is_object_or_null(&self) -> bool {
        static JSVAL_LOWER_INCL_SHIFTED_TAG_OF_OBJ_OR_NULL_SET: u64 = JSVAL_SHIFTED_TAG_NULL as u64;
        assert!((self.v >> JSVAL_TAG_SHIFT) <= JSVAL_TAG_OBJECT as u64);
        self.v >= JSVAL_LOWER_INCL_SHIFTED_TAG_OF_OBJ_OR_NULL_SET
    }

    pub fn to_object_or_null(&self) -> *mut JSObject {
        assert!(self.is_object_or_null());
        let ptrBits = self.v & JSVAL_PAYLOAD_MASK;
        assert!((ptrBits & 0x7) == 0);
        ptrBits as uint as *mut JSObject
    }

    pub fn to_private(&self) -> *c_void {
        assert!(self.is_double());
        assert!((self.v & 0x8000000000000000u64) == 0);
        (self.v << 1) as uint as *c_void
    }

    pub fn is_gcthing(&self) -> bool {
        static JSVAL_LOWER_INCL_SHIFTED_TAG_OF_GCTHING_SET: u64 = JSVAL_SHIFTED_TAG_STRING as u64;
        self.v >= JSVAL_LOWER_INCL_SHIFTED_TAG_OF_GCTHING_SET
    }

    pub fn to_gcthing(&self) -> *mut c_void {
        assert!(self.is_gcthing());
        let ptrBits = self.v & JSVAL_PAYLOAD_MASK;
        assert!((ptrBits & 0x7) == 0);
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
