const JSVAL_TAG_MAX_DOUBLE: u64 = 0x1FFF0;

const JSVAL_TYPE_DOUBLE: u64 = 0x00;
const JSVAL_TYPE_INT32: u64 = 0x01;
const JSVAL_TYPE_UNDEFINED: u64 = 0x02;
const JSVAL_TYPE_BOOLEAN: u64 = 0x03;
const JSVAL_TYPE_MAGIC: u64 = 0x04;
const JSVAL_TYPE_STRING: u64 = 0x05;
const JSVAL_TYPE_NULL: u64 = 0x06;
const JSVAL_TYPE_OBJECT: u64 = 0x07;
const JSVAL_TYPE_UNKNOWN: u64 = 0x20;

const JSVAL_TAG_OBJECT: u32 = (JSVAL_TAG_MAX_DOUBLE | JSVAL_TYPE_OBJECT) as u32;
const JSVAL_SHIFTED_TAG_OBJECT: u64 = JSVAL_TAG_OBJECT as u64 << JSVAL_TAG_SHIFT;
const JSVAL_TAG_SHIFT: int = 47;

const JSVAL_PAYLOAD_MASK: u64 = 0x00007FFFFFFFFFFF;

export INT_TO_JSVAL;
export JSVAL_TO_OBJECT;
export JSVAL_IS_PRIMITIVE;
export JSVAL_TO_PRIVATE;

#[inline(always)]
pub fn INT_TO_JSVAL(i: i32) -> JSVal {
  ((JSVAL_TAG_MAX_DOUBLE | JSVAL_TYPE_INT32) << JSVAL_TAG_SHIFT) | (i as u64)
}

#[inline(always)]
pub fn JSVAL_TO_OBJECT(v: JSVal) -> *JSObject {
  let bits = (v & JSVAL_PAYLOAD_MASK);
  assert bits & 0x7 == 0;
  bits as *JSObject
}

#[inline(always)]
pub fn JSVAL_IS_PRIMITIVE(v: JSVal) -> bool {
  v < JSVAL_SHIFTED_TAG_OBJECT
}

#[inline(always)]
pub fn JSVAL_TO_PRIVATE(v: JSVal) -> *() {
  assert v & 0x8000000000000000 == 0;
  (v << 1) as *()
}
