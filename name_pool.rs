use libc::c_char;
use vec::push;

pub struct NamePool {
    strbufs: ~[~str]
}

pub fn NamePool() -> @mut NamePool {
    @mut NamePool {
        strbufs: ~[]
    }
}

impl NamePool {
    fn add(&mut self, s: ~str) -> *c_char {
        unsafe {
            push(&mut self.strbufs, s);
            return cast::transmute(&self.strbufs[self.strbufs.len() - 1][0]);
        }
    }
}
