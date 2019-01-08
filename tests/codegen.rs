extern crate codegen;

use codegen::{Scope, Variant};

#[test]
fn empty_scope() {
    let scope = Scope::new();

    assert_eq!(scope.to_string(), "");
}

#[test]
fn single_struct() {
    let mut scope = Scope::new();

    scope.new_struct("Foo")
        .field("one", "usize")
        .field("two", "String");

    let expect = r#"
struct Foo {
    one: usize,
    two: String,
}"#;

    assert_eq!(scope.to_string(), &expect[1..]);
}

#[test]
fn empty_struct() {
    let mut scope = Scope::new();

    scope.new_struct("Foo");

    let expect = r#"
struct Foo;"#;

    assert_eq!(scope.to_string(), &expect[1..]);
}

#[test]
fn two_structs() {
    let mut scope = Scope::new();

    scope.new_struct("Foo")
        .field("one", "usize")
        .field("two", "String");

    scope.new_struct("Bar")
        .field("hello", "World");

    let expect = r#"
struct Foo {
    one: usize,
    two: String,
}

struct Bar {
    hello: World,
}"#;

    assert_eq!(scope.to_string(), &expect[1..]);
}

#[test]
fn struct_with_derive() {
    let mut scope = Scope::new();

    scope.new_struct("Foo")
        .derive("Debug").derive("Clone")
        .field("one", "usize")
        .field("two", "String");

    let expect = r#"
#[derive(Debug, Clone)]
struct Foo {
    one: usize,
    two: String,
}"#;

    assert_eq!(scope.to_string(), &expect[1..]);
}

#[test]
fn struct_with_repr() {
    let mut scope = Scope::new();

    scope.new_struct("Foo")
        .repr("C")
        .field("one", "u8")
        .field("two", "u8");

    let expect = r#"
#[repr(C)]
struct Foo {
    one: u8,
    two: u8,
}"#;

    assert_eq!(scope.to_string(), &expect[1..]);
}

#[test]
fn struct_with_allow() {
    let mut scope = Scope::new();

    scope.new_struct("Foo")
        .allow("dead_code")
        .field("one", "u8")
        .field("two", "u8");

    let expect = r#"
#[allow(dead_code)]
struct Foo {
    one: u8,
    two: u8,
}"#;

    assert_eq!(scope.to_string(), &expect[1..]);
}

#[test]
fn struct_with_generics_1() {
    let mut scope = Scope::new();

    scope.new_struct("Foo")
        .generic("T")
        .generic("U")
        .field("one", "T")
        .field("two", "U");

    let expect = r#"
struct Foo<T, U> {
    one: T,
    two: U,
}"#;

    assert_eq!(scope.to_string(), &expect[1..]);
}

#[test]
fn struct_with_generics_2() {
    let mut scope = Scope::new();

    scope.new_struct("Foo")
        .generic("T, U")
        .field("one", "T")
        .field("two", "U");

    let expect = r#"
struct Foo<T, U> {
    one: T,
    two: U,
}"#;

    assert_eq!(scope.to_string(), &expect[1..]);
}

#[test]
fn struct_with_generics_3() {
    let mut scope = Scope::new();

    scope.new_struct("Foo")
        .generic("T: Win, U")
        .field("one", "T")
        .field("two", "U");

    let expect = r#"
struct Foo<T: Win, U> {
    one: T,
    two: U,
}"#;

    assert_eq!(scope.to_string(), &expect[1..]);
}

#[test]
fn struct_where_clause_1() {
    let mut scope = Scope::new();

    scope.new_struct("Foo")
        .generic("T")
        .bound("T", "Foo")
        .field("one", "T");

    let expect = r#"
struct Foo<T>
where T: Foo,
{
    one: T,
}"#;

    assert_eq!(scope.to_string(), &expect[1..]);
}

#[test]
fn struct_where_clause_2() {
    let mut scope = Scope::new();

    scope.new_struct("Foo")
        .generic("T, U")
        .bound("T", "Foo")
        .bound("U", "Baz")
        .field("one", "T")
        .field("two", "U");

    let expect = r#"
struct Foo<T, U>
where T: Foo,
      U: Baz,
{
    one: T,
    two: U,
}"#;

    assert_eq!(scope.to_string(), &expect[1..]);
}

#[test]
fn struct_doc() {
    let mut scope = Scope::new();

    scope.new_struct("Foo")
        .doc("Hello, this is a doc string\n\
              that continues on another line.")
        .field("one", "T");

    let expect = r#"
/// Hello, this is a doc string
/// that continues on another line.
struct Foo {
    one: T,
}"#;

    assert_eq!(scope.to_string(), &expect[1..]);
}

#[test]
fn struct_in_mod() {
    let mut scope = Scope::new();

    {
        let module = scope.new_module("foo");
        module.new_struct("Foo")
            .doc("Hello some docs")
            .derive("Debug")
            .generic("T, U")
            .bound("T", "SomeBound")
            .bound("U", "SomeOtherBound")
            .field("one", "T")
            .field("two", "U")
            ;
    }

    let expect = r#"
mod foo {
    /// Hello some docs
    #[derive(Debug)]
    struct Foo<T, U>
    where T: SomeBound,
          U: SomeOtherBound,
    {
        one: T,
        two: U,
    }
}"#;

    assert_eq!(scope.to_string(), &expect[1..]);
}

#[test]
fn struct_mod_import() {
    let mut scope = Scope::new();
    scope.new_module("foo")
        .import("bar", "Bar")
        .new_struct("Foo")
        .field("bar", "Bar")
        ;

    let expect = r#"
mod foo {
    use bar::Bar;

    struct Foo {
        bar: Bar,
    }
}"#;

    assert_eq!(scope.to_string(), &expect[1..]);
}

#[test]
fn enum_with_repr() {
    let mut scope = Scope::new();

    scope.new_enum("IpAddrKind")
        .repr("u8")
        .push_variant(Variant::new("V4"))
        .push_variant(Variant::new("V6"))
        ;

    let expect = r#"
#[repr(u8)]
enum IpAddrKind {
    V4,
    V6,
}"#;

    assert_eq!(scope.to_string(), &expect[1..]);
}

#[test]
fn enum_with_allow() {
    let mut scope = Scope::new();

    scope.new_enum("IpAddrKind")
        .allow("dead_code")
        .push_variant(Variant::new("V4"))
        .push_variant(Variant::new("V6"))
        ;

    let expect = r#"
#[allow(dead_code)]
enum IpAddrKind {
    V4,
    V6,
}"#;

    assert_eq!(scope.to_string(), &expect[1..]);
}

#[test]
fn scoped_imports() {
    let mut scope = Scope::new();
    scope.new_module("foo")
        .import("bar", "Bar")
        .import("bar", "baz::Baz")
        .import("bar::quux", "quuux::Quuuux")
        .new_struct("Foo")
        .field("bar", "Bar")
        .field("baz", "baz::Baz")
        .field("quuuux", "quuux::Quuuux")
        ;

    let expect = r#"
mod foo {
    use bar::{Bar, baz};
    use bar::quux::quuux;

    struct Foo {
        bar: Bar,
        baz: baz::Baz,
        quuuux: quuux::Quuuux,
    }
}"#;

    assert_eq!(scope.to_string(), &expect[1..]);
}

#[test]
fn module_mut() {
    let mut scope = Scope::new();
    scope.new_module("foo")
        .import("bar", "Bar")
        ;

    scope.get_module_mut("foo").expect("module_mut")
        .new_struct("Foo")
        .field("bar", "Bar")
        ;

    let expect = r#"
mod foo {
    use bar::Bar;

    struct Foo {
        bar: Bar,
    }
}"#;

    assert_eq!(scope.to_string(), &expect[1..]);
}

#[test]
fn get_or_new_module() {
    let mut scope = Scope::new();
    assert!(scope.get_module("foo").is_none());

    scope.get_or_new_module("foo")
        .import("bar", "Bar")
        ;

    scope.get_or_new_module("foo")
        .new_struct("Foo")
        .field("bar", "Bar")
        ;

    let expect = r#"
mod foo {
    use bar::Bar;

    struct Foo {
        bar: Bar,
    }
}"#;

    assert_eq!(scope.to_string(), &expect[1..]);
}
