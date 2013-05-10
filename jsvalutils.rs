/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/. */

use core::libc::types::common::c95::c_void;
use glue::bindgen::*;
use jsapi::{JSVal, JSObject};

struct JSValUtils;

impl JSValUtils{
	pub fn to_object(v : &JSVal) -> *JSObject {
		unsafe { RUST_JSVAL_TO_OBJECT(*v) }
	}

	pub fn to_int(v : &JSVal) -> i32 {
		unsafe { RUST_JSVAL_TO_INT(*v) }
	}

	pub fn to_double(v : &JSVal) -> f64 {
		unsafe { RUST_JSVAL_TO_DOUBLE(*v) }
	}

	pub fn to_string(v : &JSVal) -> *c_void {
		unsafe { RUST_JSVAL_TO_STRING(*v) }
	}

	pub fn to_boolean(v: &JSVal) -> i32 {
		unsafe { RUST_JSVAL_TO_BOOLEAN(*v) }
	}

	pub fn to_gcthing(v : &JSVal) -> *c_void {
		unsafe { RUST_JSVAL_TO_GCTHING(*v) }
	}

  pub fn to_private(v : &JSVal) -> *c_void {
		unsafe { RUST_JSVAL_TO_PRIVATE(*v) }
	}

  pub fn from_private(v : *c_void) -> JSVal {
		unsafe { RUST_PRIVATE_TO_JSVAL(v) }
	}

  pub fn from_object(v : *c_void) -> JSVal {
		unsafe { RUST_OBJECT_TO_JSVAL(v) }
	}

  pub fn is_null(v : &JSVal) -> i32 {
    unsafe { RUST_JSVAL_IS_NULL(*v) }
  }

  pub fn is_void(v : &JSVal) -> i32 {
    unsafe { RUST_JSVAL_IS_VOID(*v) }
  }

  pub fn is_int(v : &JSVal) -> i32 {
    unsafe { RUST_JSVAL_IS_INT(*v) }
  }

  pub fn from_int(v : i32) -> JSVal {
    unsafe { RUST_INT_TO_JSVAL(v) }
  }

  pub fn is_double(v : &JSVal) -> i32 {
    unsafe { RUST_JSVAL_IS_DOUBLE(*v) }
  }

  pub fn from_double(v : f64) -> JSVal {
    unsafe { RUST_DOUBLE_TO_JSVAL(v) }
  }

   pub fn from_uint(v : u32) -> JSVal {
    unsafe { RUST_UINT_TO_JSVAL(v) }
  }

  pub fn is_number(v : &JSVal) -> i32 {
    unsafe { RUST_JSVAL_IS_NUMBER(*v) }
  }

   pub fn is_string(v : &JSVal) -> i32 {
    unsafe { RUST_JSVAL_IS_STRING(*v) }
  }

   pub fn from_string(v : *c_void) -> JSVal {
    unsafe { RUST_STRING_TO_JSVAL(v) }
  }

   pub fn is_object(v : &JSVal) -> i32 {
    unsafe { RUST_JSVAL_IS_OBJECT(*v) }
  }

   pub fn is_boolean(v : &JSVal) -> i32 {
    unsafe { RUST_JSVAL_IS_BOOLEAN(*v) }
  }

   pub fn from_boolean(v : i32) -> JSVal {
    unsafe { RUST_BOOLEAN_TO_JSVAL(v) }
  }

   pub fn is_primitive(v : &JSVal) -> i32 {
    unsafe { RUST_JSVAL_IS_PRIMITIVE(*v) }
  }

  pub fn is_gcthing(v : &JSVal) -> i32 {
    unsafe { RUST_JSVAL_IS_GCTHING(*v) }
  }
}
