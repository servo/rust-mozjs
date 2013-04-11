/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/. */

use core::libc::types::common::c95::c_void;
use glue::bindgen::*;
use jsapi::{JSVal, JSObject};

trait JSValUtils {
	fn to_object(&self) -> *JSObject;
  fn to_int(&self) -> i32;
  fn to_double(&self) -> f64;
  fn to_string(&self) -> *c_void;
  fn to_gcthing(&self) -> *c_void;
  fn to_boolean(&self) -> i32;
}

impl JSValUtils for JSVal {
	fn to_object(&self) -> *JSObject {
		unsafe { RUST_JSVAL_TO_OBJECT(*self) }
	}

	fn to_int(&self) -> i32 {
		unsafe { RUST_JSVAL_TO_INT(*self) }
	}

	fn to_double(&self) -> f64 {
		unsafe { RUST_JSVAL_TO_DOUBLE(*self) }
	}

	fn to_string(&self) -> *c_void {
		unsafe { RUST_JSVAL_TO_STRING(*self) }
	}

	fn to_boolean(&self) -> i32 {
		unsafe { RUST_JSVAL_TO_BOOLEAN(*self) }
	}

	fn to_gcthing(&self) -> *c_void {
		unsafe { RUST_JSVAL_TO_GCTHING(*self) }
	}

}
