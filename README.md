# rust-mozjs

Rust bindings to SpiderMonkey

[Documentation](https://doc.servo.org/mozjs/)


# Usage

Add this to your Cargo.toml:

```toml
[dependencies]
mozjs = "0.10"
```

# Example

```rust
#[macro_use]
extern crate mozjs;

use mozjs::jsapi::JS_NewGlobalObject;
use mozjs::jsapi::OnNewGlobalHookOption;
use mozjs::jsval::UndefinedValue;
use mozjs::rust::{JSEngine, RealmOptions, Runtime, SIMPLE_GLOBAL_CLASS};

use std::ptr;

fn main() {
    let engine = JSEngine::init().unwrap();
    let rt = Runtime::new(engine);
    let cx = rt.cx();

    unsafe {
        let options = RealmOptions::default();
        rooted!(in(cx) let global =
            JS_NewGlobalObject(cx, &SIMPLE_GLOBAL_CLASS, ptr::null_mut(),
                               OnNewGlobalHookOption::FireOnNewGlobalHook,
                               &*options)
        );
        rooted!(in(cx) let mut rval = UndefinedValue());
        let eval_result = rt.evaluate_script(global.handle(), "1 + 1", "test", 1, rval.handle_mut());
        match eval_result {
            Ok(_) => println!("1 + 1 from JavaScript is: {:?}", rval.get().to_int32()),
            Err(_) => println!("Something went wrong :("),
        }
    }
}

```

# Building
See https://github.com/servo/mozjs/blob/master README.md for build instructions.