use libc::c_char;
use vec::push;

export name_pool;
export methods;
export add;

type name_pool = @{
    mut strbufs: ~[~str]
};

fn name_pool() -> name_pool {
    @{mut strbufs: ~[]}
}

trait add {
    fn add(-s: ~str) -> *c_char;
}

impl name_pool : add {
    fn add(-s: ~str) -> *c_char {
        let c_str = str::as_c_str(s, |bytes| bytes);
        push(self.strbufs, s); // in theory, this should *move* the str in here..
        return c_str; // ...and so this ptr ought to be valid.
    }
}
