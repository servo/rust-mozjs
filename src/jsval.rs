/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/. */

use jsapi::{JSObject, JSString, JSGCTraceKind};
use jsapi::JSGCTraceKind::{JSTRACE_OBJECT, JSTRACE_STRING};

use libc::c_void;
use std::mem;

#[cfg(target_pointer_width = "64")]
const JSVAL_TAG_SHIFT: uint = 47u;

#[repr(u8)]
#[allow(dead_code)]
enum ValueType {
    DOUBLE              = 0x00,
    INT32               = 0x01,
    UNDEFINED           = 0x02,
    BOOLEAN             = 0x03,
    MAGIC               = 0x04,
    STRING              = 0x05,
    NULL                = 0x06,
    OBJECT              = 0x07,

    /* These never appear in a jsval; they are only provided as an out-of-band value. */
    UNKNOWN             = 0x20,
    MISSING             = 0x21
}

#[cfg(target_pointer_width = "64")]
const JSVAL_TAG_MAX_DOUBLE: u32 = 0x1FFF0u32;

#[cfg(target_pointer_width = "32")]
const JSVAL_TAG_CLEAR: u32 = 0xFFFFFF80;

#[cfg(target_pointer_width = "64")]
#[repr(u32)]
#[allow(dead_code)]
enum ValueTag {
    DOUBLE               = JSVAL_TAG_MAX_DOUBLE | (ValueType::DOUBLE as u32),
    INT32                = JSVAL_TAG_MAX_DOUBLE | (ValueType::INT32 as u32),
    UNDEFINED            = JSVAL_TAG_MAX_DOUBLE | (ValueType::UNDEFINED as u32),
    STRING               = JSVAL_TAG_MAX_DOUBLE | (ValueType::STRING as u32),
    BOOLEAN              = JSVAL_TAG_MAX_DOUBLE | (ValueType::BOOLEAN as u32),
    MAGIC                = JSVAL_TAG_MAX_DOUBLE | (ValueType::MAGIC as u32),
    NULL                 = JSVAL_TAG_MAX_DOUBLE | (ValueType::NULL as u32),
    OBJECT               = JSVAL_TAG_MAX_DOUBLE | (ValueType::OBJECT as u32),
}

#[cfg(target_pointer_width = "32")]
#[repr(u32)]
#[allow(dead_code)]
enum ValueTag {
    PRIVATE              = 0,
    INT32                = JSVAL_TAG_CLEAR as u32 | (ValueType::INT32 as u32),
    UNDEFINED            = JSVAL_TAG_CLEAR as u32 | (ValueType::UNDEFINED as u32),
    STRING               = JSVAL_TAG_CLEAR as u32 | (ValueType::STRING as u32),
    BOOLEAN              = JSVAL_TAG_CLEAR as u32 | (ValueType::BOOLEAN as u32),
    MAGIC                = JSVAL_TAG_CLEAR as u32 | (ValueType::MAGIC as u32),
    NULL                 = JSVAL_TAG_CLEAR as u32 | (ValueType::NULL as u32),
    OBJECT               = JSVAL_TAG_CLEAR as u32 | (ValueType::OBJECT as u32),
}

#[cfg(target_pointer_width = "64")]
#[repr(u64)]
#[allow(dead_code)]
enum ValueShiftedTag {
    MAX_DOUBLE   = (((JSVAL_TAG_MAX_DOUBLE as u64) << JSVAL_TAG_SHIFT) | 0xFFFFFFFFu64),
    INT32        = ((ValueTag::INT32 as u64)      << JSVAL_TAG_SHIFT),
    UNDEFINED    = ((ValueTag::UNDEFINED as u64)  << JSVAL_TAG_SHIFT),
    STRING       = ((ValueTag::STRING as u64)     << JSVAL_TAG_SHIFT),
    BOOLEAN      = ((ValueTag::BOOLEAN as u64)    << JSVAL_TAG_SHIFT),
    MAGIC        = ((ValueTag::MAGIC as u64)      << JSVAL_TAG_SHIFT),
    NULL         = ((ValueTag::NULL as u64)       << JSVAL_TAG_SHIFT),
    OBJECT       = ((ValueTag::OBJECT as u64)     << JSVAL_TAG_SHIFT)
}


#[cfg(target_pointer_width = "64")]
const JSVAL_PAYLOAD_MASK: u64 = 0x00007FFFFFFFFFFF;

// JSVal was originally type of u64.
// now this become {u64} because of the union abi issue on ARM arch. See #398.
#[deriving(PartialEq, Clone, Copy)]
pub struct JSVal {
    pub v: u64
}

#[cfg(target_pointer_width = "64")]
#[inline(always)]
fn BuildJSVal(tag: ValueTag, payload: u64) -> JSVal {
    JSVal {
        v: ((tag as u32 as u64) << JSVAL_TAG_SHIFT) | payload
    }
}

#[cfg(target_pointer_width = "32")]
#[inline(always)]
fn BuildJSVal(tag: ValueTag, payload: u64) -> JSVal {
    JSVal {
        v: ((tag as u32 as u64) << 32) | payload
    }
}

#[inline(always)]
pub fn NullValue() -> JSVal {
    BuildJSVal(ValueTag::NULL, 0)
}

#[inline(always)]
pub fn UndefinedValue() -> JSVal {
    BuildJSVal(ValueTag::UNDEFINED, 0)
}

#[inline(always)]
pub fn Int32Value(i: i32) -> JSVal {
    BuildJSVal(ValueTag::INT32, i as u32 as u64)
}

#[cfg(target_pointer_width = "64")]
#[inline(always)]
pub fn DoubleValue(f: f64) -> JSVal {
    let bits: u64 = unsafe { mem::transmute(f) };
    assert!(bits <= ValueShiftedTag::MAX_DOUBLE as u64);
    JSVal {
        v: bits
    }
}

#[cfg(target_pointer_width = "32")]
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

#[cfg(target_pointer_width = "64")]
#[inline(always)]
pub fn StringValue(s: &JSString) -> JSVal {
    let bits = s as *const JSString as uint as u64;
    assert!((bits >> JSVAL_TAG_SHIFT) == 0);
    BuildJSVal(ValueTag::STRING, bits)
}

#[cfg(target_pointer_width = "32")]
#[inline(always)]
pub fn StringValue(s: &JSString) -> JSVal {
    let bits = s as *const JSString as uint as u64;
    BuildJSVal(ValueTag::STRING, bits)
}

#[inline(always)]
pub fn BooleanValue(b: bool) -> JSVal {
    BuildJSVal(ValueTag::BOOLEAN, b as u64)
}

#[cfg(target_pointer_width = "64")]
#[inline(always)]
pub fn ObjectValue(o: &JSObject) -> JSVal {
    let bits = o as *const JSObject as uint as u64;
    assert!((bits >> JSVAL_TAG_SHIFT) == 0);
    BuildJSVal(ValueTag::OBJECT, bits)
}

#[cfg(target_pointer_width = "32")]
#[inline(always)]
pub fn ObjectValue(o: &JSObject) -> JSVal {
    let bits = o as *const JSObject as uint as u64;
    BuildJSVal(ValueTag::OBJECT, bits)
}

#[inline(always)]
pub fn ObjectOrNullValue(o: *mut JSObject) -> JSVal {
    if o.is_null() {
        NullValue()
    } else {
        ObjectValue(unsafe { &*o })
    }
}

#[cfg(target_pointer_width = "64")]
#[inline(always)]
pub fn PrivateValue(o: *const c_void) -> JSVal {
    let ptrBits = o as uint as u64;
    assert!((ptrBits & 1) == 0);
    JSVal {
        v: ptrBits >> 1
    }
}

#[cfg(target_pointer_width = "32")]
#[inline(always)]
pub fn PrivateValue(o: *const c_void) -> JSVal {
    let ptrBits = o as uint as u64;
    assert!((ptrBits & 1) == 0);
    BuildJSVal(ValueTag::PRIVATE, ptrBits)
}

impl JSVal {
    #[cfg(target_pointer_width = "64")]
    pub fn is_undefined(&self) -> bool {
        self.v == ValueShiftedTag::UNDEFINED as u64
    }

    #[cfg(target_pointer_width = "32")]
    pub fn is_undefined(&self) -> bool {
        (self.v >> 32) == ValueTag::UNDEFINED as u64
    }

    #[cfg(target_pointer_width = "64")]
    pub fn is_null(&self) -> bool {
        self.v == ValueShiftedTag::NULL as u64
    }

    #[cfg(target_pointer_width = "32")]
    pub fn is_null(&self) -> bool {
        (self.v >> 32) == ValueTag::NULL as u64
    }

    pub fn is_null_or_undefined(&self) -> bool {
        self.is_null() || self.is_undefined()
    }

    #[cfg(target_pointer_width = "64")]
    pub fn is_boolean(&self) -> bool {
        (self.v >> JSVAL_TAG_SHIFT) == ValueTag::BOOLEAN as u64
    }

    #[cfg(target_pointer_width = "32")]
    pub fn is_boolean(&self) -> bool {
        (self.v >> 32) == ValueTag::BOOLEAN as u64
    }

    #[cfg(target_pointer_width = "64")]
    pub fn is_double(&self) -> bool {
        self.v <= ValueShiftedTag::MAX_DOUBLE as u64
    }

    #[cfg(target_pointer_width = "32")]
    pub fn is_double(&self) -> bool {
        (self.v >> 32) <= JSVAL_TAG_CLEAR as u64
    }

    #[cfg(target_pointer_width = "64")]
    pub fn is_primitive(&self) -> bool {
        const JSVAL_UPPER_EXCL_SHIFTED_TAG_OF_PRIMITIVE_SET: u64 = ValueShiftedTag::OBJECT as u64;
        self.v < JSVAL_UPPER_EXCL_SHIFTED_TAG_OF_PRIMITIVE_SET
    }

    #[cfg(target_pointer_width = "32")]
    pub fn is_primitive(&self) -> bool {
        const JSVAL_UPPER_EXCL_TAG_OF_PRIMITIVE_SET: u64 = ValueTag::OBJECT as u64;
        (self.v >> 32) < JSVAL_UPPER_EXCL_TAG_OF_PRIMITIVE_SET
    }

    #[cfg(target_pointer_width = "64")]
    pub fn is_string(&self) -> bool {
        (self.v >> JSVAL_TAG_SHIFT) == ValueTag::STRING as u64
    }

    #[cfg(target_pointer_width = "32")]
    pub fn is_string(&self) -> bool {
        (self.v >> 32) == ValueTag::STRING as u64
    }

    #[cfg(target_pointer_width = "64")]
    pub fn is_object(&self) -> bool {
        assert!((self.v >> JSVAL_TAG_SHIFT) <= ValueTag::OBJECT as u64);
        self.v >= ValueShiftedTag::OBJECT as u64
    }

    #[cfg(target_pointer_width = "32")]
    pub fn is_object(&self) -> bool {
        (self.v >> 32) == ValueTag::OBJECT as u64
    }

    #[cfg(target_pointer_width = "64")]
    pub fn to_boolean(&self) -> bool {
        assert!(self.is_boolean());
        (self.v & JSVAL_PAYLOAD_MASK) != 0
    }

    #[cfg(target_pointer_width = "32")]
    pub fn to_boolean(&self) -> bool {
        (self.v & 0x00000000FFFFFFFF) != 0
    }

    pub fn to_object(&self) -> *mut JSObject {
        assert!(self.is_object());
        self.to_object_or_null()
    }

    #[cfg(target_pointer_width = "64")]
    pub fn is_object_or_null(&self) -> bool {
        const JSVAL_LOWER_INCL_SHIFTED_TAG_OF_OBJ_OR_NULL_SET: u64 = ValueShiftedTag::NULL as u64;
        assert!((self.v >> JSVAL_TAG_SHIFT) <= ValueTag::OBJECT as u64);
        self.v >= JSVAL_LOWER_INCL_SHIFTED_TAG_OF_OBJ_OR_NULL_SET
    }

    #[cfg(target_pointer_width = "32")]
    pub fn is_object_or_null(&self) -> bool {
        const JSVAL_LOWER_INCL_TAG_OF_OBJ_OR_NULL_SET: u64 = ValueTag::NULL as u64;
        assert!((self.v >> 32) <= ValueTag::OBJECT as u64);
        (self.v >> 32) >= JSVAL_LOWER_INCL_TAG_OF_OBJ_OR_NULL_SET
    }

    #[cfg(target_pointer_width = "64")]
    pub fn to_object_or_null(&self) -> *mut JSObject {
        assert!(self.is_object_or_null());
        let ptrBits = self.v & JSVAL_PAYLOAD_MASK;
        assert!((ptrBits & 0x7) == 0);
        ptrBits as uint as *mut JSObject
    }

    #[cfg(target_pointer_width = "32")]
    pub fn to_object_or_null(&self) -> *mut JSObject {
        assert!(self.is_object_or_null());
        let ptrBits: u32 = (self.v & 0x00000000FFFFFFFF) as u32;
        ptrBits as *mut JSObject
    }

    #[cfg(target_pointer_width = "64")]
    pub fn to_private(&self) -> *const c_void {
        assert!(self.is_double());
        assert!((self.v & 0x8000000000000000u64) == 0);
        (self.v << 1) as uint as *const c_void
    }

    #[cfg(target_pointer_width = "32")]
    pub fn to_private(&self) -> *const c_void {
        let ptrBits: u32 = (self.v & 0x00000000FFFFFFFF) as u32;
        ptrBits as *const c_void
    }

    #[cfg(target_pointer_width = "64")]
    pub fn is_gcthing(&self) -> bool {
        const JSVAL_LOWER_INCL_SHIFTED_TAG_OF_GCTHING_SET: u64 = ValueShiftedTag::STRING as u64;
        self.v >= JSVAL_LOWER_INCL_SHIFTED_TAG_OF_GCTHING_SET
    }

    #[cfg(target_pointer_width = "32")]
    pub fn is_gcthing(&self) -> bool {
        const JSVAL_LOWER_INCL_TAG_OF_GCTHING_SET: u64 = ValueTag::STRING as u64;
        (self.v >> 32) >= JSVAL_LOWER_INCL_TAG_OF_GCTHING_SET
    }

    #[cfg(target_pointer_width = "64")]
    pub fn to_gcthing(&self) -> *mut c_void {
        assert!(self.is_gcthing());
        let ptrBits = self.v & JSVAL_PAYLOAD_MASK;
        assert!((ptrBits & 0x7) == 0);
        ptrBits as *mut c_void
    }

    #[cfg(target_pointer_width = "32")]
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
