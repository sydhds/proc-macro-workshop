# Proc macro debug

## Macro backtrace

Note: use when some code in the proc macro panic (and you need to know where)

RUST_BACKTRACE=full RUSTFLAGS=-Zproc-macro-backtrace cargo +nightly run

## Macro diagnostic

Add feature: 

```rust
#![feature(proc_macro_diagnostic)]
```

and add code like: 

```rust
if start_v >= end_v {
    start.span()
         .unwrap()
         .warning(format!("Require start value ({}) to be lower than end value ({})", start_v, end_v))
         .emit();
}
```

or 

```rust
if start_v >= end_v {
    start.span()
         .unwrap()
         .error(format!("Require start value ({}) to be lower than end value ({})", start_v, end_v))
         .emit();
}
```


Compile/Run/Test/Expand with nightly:

```commandline
cargo +nightly expand
```

## trace_macro!

* https://doc.rust-lang.org/nightly/unstable-book/library-features/trace-macros.html

