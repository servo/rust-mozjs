/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/. */

use jsapi::{JSVal, JSObject};

static JSVAL_TAG_MAX_DOUBLE: u64 = 0x1FFF0;

static JSVAL_TYPE_DOUBLE: u64 = 0x00;
static JSVAL_TYPE_INT32: u64 = 0x01;
static JSVAL_TYPE_UNDEFINED: u64 = 0x02;
static JSVAL_TYPE_BOOLEAN: u64 = 0x03;
static JSVAL_TYPE_MAGIC: u64 = 0x04;
static JSVAL_TYPE_STRING: u64 = 0x05;
static JSVAL_TYPE_NULL: u64 = 0x06;
static JSVAL_TYPE_OBJECT: u64 = 0x07;
static JSVAL_TYPE_UNKNOWN: u64 = 0x20;

static JSVAL_TAG_OBJECT: u32 = (JSVAL_TAG_MAX_DOUBLE | JSVAL_TYPE_OBJECT) as u32;
static JSVAL_SHIFTED_TAG_OBJECT: u64 = JSVAL_TAG_OBJECT as u64 << JSVAL_TAG_SHIFT;
static JSVAL_TAG_SHIFT: int = 47;

static JSVAL_PAYLOAD_MASK: u64 = 0x00007FFFFFFFFFFF;

#[inline(always)]
pub fn INT_TO_JSVAL(i: i32) -> JSVal {
    JSVal {
        v: ((JSVAL_TAG_MAX_DOUBLE | JSVAL_TYPE_INT32) << JSVAL_TAG_SHIFT) | (i as u64)
    }
}

#[inline(always)]
pub fn JSVAL_TO_OBJECT(v: JSVal) -> *JSObject {
    let bits = (v.v & JSVAL_PAYLOAD_MASK);
    assert!(bits & 0x7 == 0);
    bits as *JSObject
}

#[inline(always)]
pub fn JSVAL_IS_PRIMITIVE(v: JSVal) -> bool {
    v.v < JSVAL_SHIFTED_TAG_OBJECT
}

#[inline(always)]
pub fn JSVAL_IS_OBJECT(v: JSVal) -> bool {
    v.v >= JSVAL_SHIFTED_TAG_OBJECT
}

#[inline(always)]
pub fn JSVAL_TO_PRIVATE(v: JSVal) -> *() {
    assert!(v.v & 0x8000000000000000 == 0);
    (v.v << 1) as *()
}
