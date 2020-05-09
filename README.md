# Codegen

Provides an builder API to assist in generating Rust code.

[![Build Status](https://travis-ci.org/carllerche/codegen.svg?branch=master)](https://travis-ci.org/carllerche/codegen)

More information about this crate can be found in the [crate documentation][dox]

[dox]: https://docs.rs/codegen/0.1.3/codegen/

## Installation

To use `codegen`, first add this to your `Cargo.toml`:

```toml
[dependencies]
codegen = "0.1.3"
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

## License

This project is licensed under the [MIT license](LICENSE).

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in `codegen` by you, shall be licensed as MIT, without any
additional terms or conditions.
