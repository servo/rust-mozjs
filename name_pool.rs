use libc::c_char;
use vec::push;

pub type name_pool = @{
    mut strbufs: ~[~str]
};

pub fn name_pool() -> name_pool {
    @{mut strbufs: ~[]}
}

pub trait add {
    fn add(-s: ~str) -> *c_char;
}

impl name_pool : add {
    fn add(-s: ~str) -> *c_char {
        let c_str = str::as_c_str(s, |bytes| bytes);
        push(&mut self.strbufs, s); // in theory, this should *move* the str in here..
        return c_str; // ...and so this ptr ought to be valid.
    }
}
