use core::libc::c_char;
use core::vec::push;

pub struct NamePool {
    strbufs: ~[~[u8]]
}

pub fn NamePool() -> @mut NamePool {
    @mut NamePool {
        strbufs: ~[]
    }
}

pub impl NamePool {
    fn add(&mut self, s: ~str) -> *c_char {
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
