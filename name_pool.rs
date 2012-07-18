import libc::c_char;
import vec::push;

type name_pool = @{
    mut strbufs: ~[~str]
};

fn name_pool() -> name_pool {
    @{mut strbufs: ~[]}
}

trait add {
    fn add(-s: ~str) -> *c_char;
}

impl methods of add for name_pool {
    fn add(-s: ~str) -> *c_char {
        let c_str = str::as_c_str(s, |bytes| bytes);
        push(self.strbufs, s); // in theory, this should *move* the str in here..
        ret c_str; // ...and so this ptr ought to be valid.
    }
}
