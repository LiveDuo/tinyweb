
pub mod callbacks;
pub mod allocations;
pub mod runtime;
pub mod invoke;

pub mod signals;
pub mod element;
pub mod router;

// Use: crate::println!("{}", 42);
#[macro_export]
macro_rules! println {
    ($fmt:expr) => { Js::invoke("console.log({})", &[format!($fmt).into()]); };
    ($fmt:expr, $($arg:tt)*) => { Js::invoke("console.log({})", &[format!($fmt, $($arg)*).into()]); };
}

// Web browser specification
// https://github.com/w3c/webref

// Count LOC (excluding tests)
// ```
// git ls-files ':(glob)src/rust/src/**' | xargs cat | sed '/#\[test\]/,/}/d' | wc -l
// ```

// List files
// ```
// git ls-files ':(glob)src/rust/src/**' | xargs wc -l | sort -r
// ```
