/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::cast;
use std::libc::c_char;
use std::uint;
use std::vec::push;

pub struct NamePool {
    strbufs: ~[~[u8]]
}

pub fn NamePool() -> @mut NamePool {
    @mut NamePool {
        strbufs: ~[]
    }
}

impl NamePool {
    pub fn add(&mut self, s: ~str) -> *c_char {
        unsafe {
            let mut strbuf = ~[];
            for uint::range(0, s.len()) |i| {
                strbuf.push(s[i]);
            }
            strbuf.push(0);

            push(&mut self.strbufs, strbuf);
            return cast::transmute(&self.strbufs[self.strbufs.len() - 1][0]);
        }
    }
}
