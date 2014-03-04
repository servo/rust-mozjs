/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use jsapi::JSTracer;

use serialize::Encoder;

impl Encoder for JSTracer {
    fn emit_nil(&mut self) {}
    fn emit_uint(&mut self, _v: uint) {}
    fn emit_u64(&mut self, _v: u64) {}
    fn emit_u32(&mut self, __v: u32) {}
    fn emit_u16(&mut self, _v: u16) {}
    fn emit_u8(&mut self, _v: u8) {}
    fn emit_int(&mut self, _v: int) {}
    fn emit_i64(&mut self, _v: i64) {}
    fn emit_i32(&mut self, _v: i32) {}
    fn emit_i16(&mut self, _v: i16) {}
    fn emit_i8(&mut self, _v: i8) {}
    fn emit_bool(&mut self, _v: bool) {}
    fn emit_f64(&mut self, _v: f64) {}
    fn emit_f32(&mut self, _v: f32) {}
    fn emit_char(&mut self, _v: char) {}
    fn emit_str(&mut self, _v: &str) {}
    fn emit_enum(&mut self, _name: &str, f: |&mut JSTracer|) {
        f(self);
    }
    fn emit_enum_variant(&mut self, _v_name: &str, _v_id: uint, _len: uint, f: |&mut JSTracer|) {
        f(self);
    }
    fn emit_enum_variant_arg(&mut self, _a_idx: uint, f: |&mut JSTracer|) {
        f(self);
    }
    fn emit_enum_struct_variant(&mut self, _v_name: &str, _v_id: uint, _len: uint, f: |&mut JSTracer|) {
        f(self);
    }
    fn emit_enum_struct_variant_field(&mut self, _f_name: &str, _f_idx: uint, f: |&mut JSTracer|) {
        f(self);
    }
    fn emit_struct(&mut self, _name: &str, _len: uint, f: |&mut JSTracer|) {
        f(self);
    }
    fn emit_struct_field(&mut self, _f_name: &str, _f_idx: uint, f: |&mut JSTracer|) {
        f(self);
    }
    fn emit_tuple(&mut self, _len: uint, f: |&mut JSTracer|) {
        f(self);
    }
    fn emit_tuple_arg(&mut self, _idx: uint, f: |&mut JSTracer|) {
        f(self);
    }
    fn emit_tuple_struct(&mut self, _name: &str, _len: uint, f: |&mut JSTracer|) {
        f(self);
    }
    fn emit_tuple_struct_arg(&mut self, _f_idx: uint, f: |&mut JSTracer|) {
        f(self);
    }
    fn emit_option(&mut self, f: |&mut JSTracer|) {
        f(self);
    }
    fn emit_option_none(&mut self) {}
    fn emit_option_some(&mut self, f: |&mut JSTracer|) {
        f(self);
    }
    fn emit_seq(&mut self, _len: uint, f: |this: &mut JSTracer|) {
        f(self);
    }
    fn emit_seq_elt(&mut self, _idx: uint, f: |this: &mut JSTracer|) {
        f(self);
    }
    fn emit_map(&mut self, _len: uint, f: |&mut JSTracer|) {
        f(self);
    }
    fn emit_map_elt_key(&mut self, _idx: uint, f: |&mut JSTracer|) {
        f(self);
    }
    fn emit_map_elt_val(&mut self, _idx: uint, f: |&mut JSTracer|) {
        f(self);
    }
}