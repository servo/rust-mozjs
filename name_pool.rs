use libc::c_char;
use vec::push;

pub type NamePool = @{
    mut strbufs: ~[~str]
};

pub fn NamePool() -> NamePool {
    @{mut strbufs: ~[]}
}

pub trait add {
    fn add(-s: ~str) -> *c_char;
}

impl NamePool : add {
    fn add(-s: ~str) -> *c_char {
        let c_str = str::as_c_str(s, |bytes| bytes);
        push(&mut self.strbufs, move s); // in theory, this should *move* the str in here..
        return c_str; // ...and so this ptr ought to be valid.
    }
}
