/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Conversions of Rust values to and from `JSVal`.
//!
//! | IDL type                | Type                             |
//! |-------------------------|----------------------------------|
//! | any                     | `JSVal`                          |
//! | boolean                 | `bool`                           |
//! | byte                    | `i8`                             |
//! | octet                   | `u8`                             |
//! | short                   | `i16`                            |
//! | unsigned short          | `u16`                            |
//! | long                    | `i32`                            |
//! | unsigned long           | `u32`                            |
//! | long long               | `i64`                            |
//! | unsigned long long      | `u64`                            |
//! | unrestricted float      | `f32`                            |
//! | float                   | `Finite<f32>`                    |
//! | unrestricted double     | `f64`                            |
//! | double                  | `Finite<f64>`                    |
//! | USVString               | `String`                         |
//! | object                  | `*mut JSObject`                  |
//! | nullable types          | `Option<T>`                      |
//! | sequences               | `Vec<T>`                         |

#![deny(missing_docs)]

use error::throw_type_error;
use glue::RUST_JS_NumberValue;
use jsapi::JSPROP_ENUMERATE;
use jsapi::{JSContext, JSObject, JSString, HandleValue, MutableHandleValue};
use jsapi::{JS_NewUCStringCopyN, JS_StringHasLatin1Chars, JS_WrapValue};
use jsapi::{JS_GetLatin1StringCharsAndLength, JS_GetTwoByteStringCharsAndLength};
use jsapi::{JS_NewArrayObject1, JS_DefineElement, RootedValue, RootedObject};
use jsapi::{JS_GetArrayLength, JS_GetElement};
use jsval::{BooleanValue, Int32Value, NullValue, UInt32Value, UndefinedValue};
use jsval::{JSVal, ObjectValue, ObjectOrNullValue, StringValue};
use rust::{ToBoolean, ToNumber, ToUint16, ToInt32, ToUint32, ToInt64, ToUint64, ToString};
use libc;
use num::Float;
use num::traits::{Bounded, Zero};
use std::rc::Rc;
use std::{ptr, slice};

trait As<O>: Copy {
    fn cast(self) -> O;
}

macro_rules! impl_as {
    ($I:ty, $O:ty) => (
        impl As<$O> for $I {
            fn cast(self) -> $O {
                self as $O
            }
        }
    )
}

impl_as!(f64, u8);
impl_as!(f64, u16);
impl_as!(f64, u32);
impl_as!(f64, u64);
impl_as!(f64, i8);
impl_as!(f64, i16);
impl_as!(f64, i32);
impl_as!(f64, i64);

impl_as!(u8, f64);
impl_as!(u16, f64);
impl_as!(u32, f64);
impl_as!(u64, f64);
impl_as!(i8, f64);
impl_as!(i16, f64);
impl_as!(i32, f64);
impl_as!(i64, f64);

impl_as!(i32, i8);
impl_as!(i32, u8);
impl_as!(i32, i16);
impl_as!(u16, u16);
impl_as!(i32, i32);
impl_as!(u32, u32);
impl_as!(i64, i64);
impl_as!(u64, u64);

/// A trait to convert Rust types to `JSVal`s.
pub trait ToJSValConvertible {
    /// Convert `self` to a `JSVal`. JSAPI failure causes a panic.
    unsafe fn to_jsval(&self, cx: *mut JSContext, rval: MutableHandleValue);
}

/// A trait to convert `JSVal`s to Rust types.
pub trait FromJSValConvertible: Sized {
    /// Optional configurable behaviour switch; use () for no configuration.
    type Config;
    /// Convert `val` to type `Self`.
    /// Optional configuration of type `T` can be passed as the `option`
    /// argument.
    /// If it returns `Err(())`, a JSAPI exception is pending.
    unsafe fn from_jsval(cx: *mut JSContext,
                         val: HandleValue,
                         option: Self::Config)
                         -> Result<Self, ()>;
}

/// Behavior for converting out-of-range integers.
#[derive(PartialEq, Eq, Clone)]
pub enum ConversionBehavior {
    /// Wrap into the integer's range.
    Default,
    /// Throw an exception.
    EnforceRange,
    /// Clamp into the integer's range.
    Clamp,
}

/// Try to cast the number to a smaller type, but
/// if it doesn't fit, it will return an error.
unsafe fn enforce_range<D>(cx: *mut JSContext, d: f64) -> Result<D, ()>
    where D: Bounded + As<f64>,
          f64: As<D>
{
    if d.is_infinite() {
        throw_type_error(cx, "value out of range in an EnforceRange argument");
        return Err(());
    }

    let rounded = d.round();
    if D::min_value().cast() <= rounded && rounded <= D::max_value().cast() {
        Ok(rounded.cast())
    } else {
        throw_type_error(cx, "value out of range in an EnforceRange argument");
        Err(())
    }
}

/// Try to cast the number to a smaller type, but if it doesn't fit,
/// round it to the MAX or MIN of the source type before casting it to
/// the destination type.
fn clamp_to<D>(d: f64) -> D
    where D: Bounded + As<f64> + Zero,
          f64: As<D>
{
    if d.is_nan() {
        D::zero()
    } else if d > D::max_value().cast() {
        D::max_value()
    } else if d < D::min_value().cast() {
        D::min_value()
    } else {
        d.cast()
    }
}

// https://heycam.github.io/webidl/#es-void
impl ToJSValConvertible for () {
    unsafe fn to_jsval(&self, _cx: *mut JSContext, rval: MutableHandleValue) {
        rval.set(UndefinedValue());
    }
}

impl ToJSValConvertible for JSVal {
    unsafe fn to_jsval(&self, cx: *mut JSContext, rval: MutableHandleValue) {
        rval.set(*self);
        if !JS_WrapValue(cx, rval) {
            panic!("JS_WrapValue failed.");
        }
    }
}

impl ToJSValConvertible for HandleValue {
    unsafe fn to_jsval(&self, cx: *mut JSContext, rval: MutableHandleValue) {
        rval.set(self.get());
        if !JS_WrapValue(cx, rval) {
            panic!("JS_WrapValue failed.");
        }
    }
}

#[inline]
unsafe fn convert_int_from_jsval<T, M>(cx: *mut JSContext, value: HandleValue,
                                       option: ConversionBehavior,
                                       convert_fn: unsafe fn(*mut JSContext, HandleValue) -> Result<M, ()>)
                                       -> Result<T, ()>
    where T: Bounded + Zero + As<f64>,
          M: Zero + As<T>,
          f64: As<T>
{
    match option {
        ConversionBehavior::Default => Ok(try!(convert_fn(cx, value)).cast()),
        ConversionBehavior::EnforceRange => enforce_range(cx, try!(ToNumber(cx, value))),
        ConversionBehavior::Clamp => Ok(clamp_to(try!(ToNumber(cx, value)))),
    }
}

// https://heycam.github.io/webidl/#es-boolean
impl ToJSValConvertible for bool {
    unsafe fn to_jsval(&self, _cx: *mut JSContext, rval: MutableHandleValue) {
        rval.set(BooleanValue(*self));
    }
}

// https://heycam.github.io/webidl/#es-boolean
impl FromJSValConvertible for bool {
    type Config = ();
    unsafe fn from_jsval(_cx: *mut JSContext, val: HandleValue, _option: ()) -> Result<bool, ()> {
        Ok(ToBoolean(val))
    }
}

// https://heycam.github.io/webidl/#es-byte
impl ToJSValConvertible for i8 {
    unsafe fn to_jsval(&self, _cx: *mut JSContext, rval: MutableHandleValue) {
        rval.set(Int32Value(*self as i32));
    }
}

// https://heycam.github.io/webidl/#es-byte
impl FromJSValConvertible for i8 {
    type Config = ConversionBehavior;
    unsafe fn from_jsval(cx: *mut JSContext,
                         val: HandleValue,
                         option: ConversionBehavior)
                         -> Result<i8, ()> {
        convert_int_from_jsval(cx, val, option, ToInt32)
    }
}

// https://heycam.github.io/webidl/#es-octet
impl ToJSValConvertible for u8 {
    unsafe fn to_jsval(&self, _cx: *mut JSContext, rval: MutableHandleValue) {
        rval.set(Int32Value(*self as i32));
    }
}

// https://heycam.github.io/webidl/#es-octet
impl FromJSValConvertible for u8 {
    type Config = ConversionBehavior;
    unsafe fn from_jsval(cx: *mut JSContext,
                         val: HandleValue,
                         option: ConversionBehavior)
                         -> Result<u8, ()> {
        convert_int_from_jsval(cx, val, option, ToInt32)
    }
}

// https://heycam.github.io/webidl/#es-short
impl ToJSValConvertible for i16 {
    unsafe fn to_jsval(&self, _cx: *mut JSContext, rval: MutableHandleValue) {
        rval.set(Int32Value(*self as i32));
    }
}

// https://heycam.github.io/webidl/#es-short
impl FromJSValConvertible for i16 {
    type Config = ConversionBehavior;
    unsafe fn from_jsval(cx: *mut JSContext,
                         val: HandleValue,
                         option: ConversionBehavior)
                         -> Result<i16, ()> {
        convert_int_from_jsval(cx, val, option, ToInt32)
    }
}

// https://heycam.github.io/webidl/#es-unsigned-short
impl ToJSValConvertible for u16 {
    unsafe fn to_jsval(&self, _cx: *mut JSContext, rval: MutableHandleValue) {
        rval.set(Int32Value(*self as i32));
    }
}

// https://heycam.github.io/webidl/#es-unsigned-short
impl FromJSValConvertible for u16 {
    type Config = ConversionBehavior;
    unsafe fn from_jsval(cx: *mut JSContext,
                         val: HandleValue,
                         option: ConversionBehavior)
                         -> Result<u16, ()> {
        convert_int_from_jsval(cx, val, option, ToUint16)
    }
}

// https://heycam.github.io/webidl/#es-long
impl ToJSValConvertible for i32 {
    unsafe fn to_jsval(&self, _cx: *mut JSContext, rval: MutableHandleValue) {
        rval.set(Int32Value(*self));
    }
}

// https://heycam.github.io/webidl/#es-long
impl FromJSValConvertible for i32 {
    type Config = ConversionBehavior;
    unsafe fn from_jsval(cx: *mut JSContext,
                         val: HandleValue,
                         option: ConversionBehavior)
                         -> Result<i32, ()> {
        convert_int_from_jsval(cx, val, option, ToInt32)
    }
}

// https://heycam.github.io/webidl/#es-unsigned-long
impl ToJSValConvertible for u32 {
    unsafe fn to_jsval(&self, _cx: *mut JSContext, rval: MutableHandleValue) {
        rval.set(UInt32Value(*self));
    }
}

// https://heycam.github.io/webidl/#es-unsigned-long
impl FromJSValConvertible for u32 {
    type Config = ConversionBehavior;
    unsafe fn from_jsval(cx: *mut JSContext,
                         val: HandleValue,
                         option: ConversionBehavior)
                         -> Result<u32, ()> {
        convert_int_from_jsval(cx, val, option, ToUint32)
    }
}

// https://heycam.github.io/webidl/#es-long-long
impl ToJSValConvertible for i64 {
    unsafe fn to_jsval(&self, _cx: *mut JSContext, rval: MutableHandleValue) {
        rval.set(RUST_JS_NumberValue(*self as f64));
    }
}

// https://heycam.github.io/webidl/#es-long-long
impl FromJSValConvertible for i64 {
    type Config = ConversionBehavior;
    unsafe fn from_jsval(cx: *mut JSContext,
                         val: HandleValue,
                         option: ConversionBehavior)
                         -> Result<i64, ()> {
        convert_int_from_jsval(cx, val, option, ToInt64)
    }
}

// https://heycam.github.io/webidl/#es-unsigned-long-long
impl ToJSValConvertible for u64 {
    unsafe fn to_jsval(&self, _cx: *mut JSContext, rval: MutableHandleValue) {
        rval.set(RUST_JS_NumberValue(*self as f64));
    }
}

// https://heycam.github.io/webidl/#es-unsigned-long-long
impl FromJSValConvertible for u64 {
    type Config = ConversionBehavior;
    unsafe fn from_jsval(cx: *mut JSContext,
                         val: HandleValue,
                         option: ConversionBehavior)
                         -> Result<u64, ()> {
        convert_int_from_jsval(cx, val, option, ToUint64)
    }
}

// https://heycam.github.io/webidl/#es-float
impl ToJSValConvertible for f32 {
    unsafe fn to_jsval(&self, _cx: *mut JSContext, rval: MutableHandleValue) {
        rval.set(RUST_JS_NumberValue(*self as f64));
    }
}

// https://heycam.github.io/webidl/#es-float
impl FromJSValConvertible for f32 {
    type Config = ();
    unsafe fn from_jsval(cx: *mut JSContext, val: HandleValue, _option: ()) -> Result<f32, ()> {
        let result = ToNumber(cx, val);
        result.map(|f| f as f32)
    }
}

// https://heycam.github.io/webidl/#es-double
impl ToJSValConvertible for f64 {
    unsafe fn to_jsval(&self, _cx: *mut JSContext, rval: MutableHandleValue) {
        rval.set(RUST_JS_NumberValue(*self));
    }
}

// https://heycam.github.io/webidl/#es-double
impl FromJSValConvertible for f64 {
    type Config = ();
    unsafe fn from_jsval(cx: *mut JSContext, val: HandleValue, _option: ()) -> Result<f64, ()> {
        ToNumber(cx, val)
    }
}

/// Converts a `JSString`, encoded in "Latin1" (i.e. U+0000-U+00FF encoded as 0x00-0xFF) into a
/// `String`.
pub unsafe fn latin1_to_string(cx: *mut JSContext, s: *mut JSString) -> String {
    assert!(JS_StringHasLatin1Chars(s));

    let mut length = 0;
    let chars = JS_GetLatin1StringCharsAndLength(cx, ptr::null(), s, &mut length);
    assert!(!chars.is_null());

    let chars = slice::from_raw_parts(chars, length as usize);
    chars.iter().map(|&c| c as char).collect()
}

// https://heycam.github.io/webidl/#es-USVString
impl ToJSValConvertible for str {
    unsafe fn to_jsval(&self, cx: *mut JSContext, rval: MutableHandleValue) {
        let mut string_utf16: Vec<u16> = Vec::with_capacity(self.len());
        string_utf16.extend(self.encode_utf16());
        let jsstr = JS_NewUCStringCopyN(cx,
                                        string_utf16.as_ptr(),
                                        string_utf16.len() as libc::size_t);
        if jsstr.is_null() {
            panic!("JS_NewUCStringCopyN failed");
        }
        rval.set(StringValue(&*jsstr));
    }
}

// https://heycam.github.io/webidl/#es-USVString
impl ToJSValConvertible for String {
    unsafe fn to_jsval(&self, cx: *mut JSContext, rval: MutableHandleValue) {
        (**self).to_jsval(cx, rval);
    }
}

// https://heycam.github.io/webidl/#es-USVString
impl FromJSValConvertible for String {
    type Config = ();
    unsafe fn from_jsval(cx: *mut JSContext, value: HandleValue, _: ()) -> Result<String, ()> {
        let jsstr = ToString(cx, value);
        if jsstr.is_null() {
            debug!("ToString failed");
            return Err(());
        }
        if JS_StringHasLatin1Chars(jsstr) {
            return Ok(latin1_to_string(cx, jsstr));
        }

        let mut length = 0;
        let chars = JS_GetTwoByteStringCharsAndLength(cx, ptr::null(), jsstr, &mut length);
        assert!(!chars.is_null());
        let char_vec = slice::from_raw_parts(chars, length as usize);
        Ok(String::from_utf16_lossy(char_vec))
    }
}

impl<T: ToJSValConvertible> ToJSValConvertible for Option<T> {
    unsafe fn to_jsval(&self, cx: *mut JSContext, rval: MutableHandleValue) {
        match self {
            &Some(ref value) => value.to_jsval(cx, rval),
            &None => rval.set(NullValue()),
        }
    }
}

impl<T: ToJSValConvertible> ToJSValConvertible for Rc<T> {
    unsafe fn to_jsval(&self, cx: *mut JSContext, rval: MutableHandleValue) {
        (**self).to_jsval(cx, rval)
    }
}

impl<T: FromJSValConvertible> FromJSValConvertible for Option<T> {
    type Config = T::Config;
    unsafe fn from_jsval(cx: *mut JSContext,
                         value: HandleValue,
                         option: T::Config)
                         -> Result<Option<T>, ()> {
        if value.get().is_null_or_undefined() {
            Ok(None)
        } else {
            let result: Result<T, ()> = FromJSValConvertible::from_jsval(cx, value, option);
            result.map(Some)
        }
    }
}

// https://heycam.github.io/webidl/#es-sequence
impl<T: ToJSValConvertible> ToJSValConvertible for Vec<T> {
    unsafe fn to_jsval(&self, cx: *mut JSContext, rval: MutableHandleValue) {
        let js_array = RootedObject::new(cx, JS_NewArrayObject1(cx, self.len() as libc::size_t));
        assert!(!js_array.handle().is_null());

        let mut val = RootedValue::new(cx, UndefinedValue());
        for (index, obj) in self.iter().enumerate() {
            obj.to_jsval(cx, val.handle_mut());

            assert!(JS_DefineElement(cx, js_array.handle(),
                                     index as u32, val.handle(), JSPROP_ENUMERATE, None, None));
        }

        rval.set(ObjectValue(&*js_array.handle().get()));
    }
}

impl<C: Clone, T: FromJSValConvertible<Config=C>> FromJSValConvertible for Vec<T> {
    type Config = C;

    unsafe fn from_jsval(cx: *mut JSContext,
                         value: HandleValue,
                         option: C)
                         -> Result<Vec<T>, ()> {
        let mut length = 0;

        if !value.is_object() {
            throw_type_error(cx, "Non objects cannot be converted to sequence");
            return Err(())
        }

        let obj = RootedObject::new(cx, value.to_object());
        if JS_GetArrayLength(cx, obj.handle(), &mut length) {
            let mut ret = Vec::with_capacity(length as usize);

            for i in 0..length {
                let mut val = RootedValue::new(cx, UndefinedValue());
                if !JS_GetElement(cx, obj.handle(), i, val.handle_mut()) {
                    // On JS Exception return Err
                    return Err(());
                }
                ret.push(try!(T::from_jsval(cx, val.handle(), option.clone())));
            }

            Ok(ret)
        } else {
            Err(())
        }
    }
}

// https://heycam.github.io/webidl/#es-object
impl ToJSValConvertible for *mut JSObject {
    unsafe fn to_jsval(&self, cx: *mut JSContext, rval: MutableHandleValue) {
        rval.set(ObjectOrNullValue(*self));
        assert!(JS_WrapValue(cx, rval));
    }
}
