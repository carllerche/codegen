# Codegen

Provides an builder API to assist in generating Rust code.

## Installation

To use `codegen`, first add this to your `Cargo.toml`:

```toml
[dependencies]
codegen = { git = "https://github.com/carllerche/codegen" } # Soon on crates.io
```

Next, add this to your crate:

```rust
extern crate codegen;
```

## Usage

1) Create a `Scope` instance.
2) Use the builder API to add elements to the scope.
3) Call `Scope::to_string()` to get the generated code.

For example:

```rust
use codegen::Scope;

let mut scope = Scope::new();

scope.new_struct("Foo")
    .derive("Debug")
    .field("one", "usize")
    .field("two", "String");

println!("{}", scope.to_string());
```

## Non-goals

`codegen` will not attempt to perform anything beyond basic formatting. For
improved formatting, the generated code can be passed to `rustfmt`.
