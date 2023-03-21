use fromsuper::FromSuper;

use std::collections::HashMap;
use std::rc::Rc;

#[derive(Clone)]
struct Bar {
    bar: Option<u32>,
    baz: Option<Option<String>>,
}

#[derive(PartialEq, Eq, Debug, FromSuper)]
#[fromsuper(from_type = "crate::Bar", unpack = true)]
struct Foo {
    bar: u32,
    #[fromsuper(no_unpack = false)]
    baz: Option<String>,
}

#[test]
fn basic_unwrap() {
    assert_eq!(
        Foo { bar: 42, baz: None },
        Foo::try_from(Bar {
            bar: Some(42),
            baz: Some(None),
        })
        .unwrap()
    );

    assert!(Foo::try_from(Bar {
        bar: Some(42),
        baz: None,
    })
    .is_err());
}

#[test]
fn basic_try_unwrap() {
    assert!(Foo::try_from(Bar {
        bar: Some(42),
        baz: None,
    })
    .is_err());

    assert!(Foo::try_from(Bar {
        bar: Some(42),
        baz: Some(None),
    })
    .is_ok());
}

struct BarGen<T> {
    x: Option<Vec<T>>,
}

#[derive(FromSuper)]
#[fromsuper(from_type = "BarGen<T>", unpack = true)]
struct FooGen<T> {
    x: Vec<T>,
}

#[test]
fn test_generics_single() {
    let bar = BarGen {
        x: Some(vec!["abc"]),
    };

    let foo = FooGen::try_from(bar).unwrap();
    assert_eq!(foo.x[0], "abc")
}

struct BarGenMultiNoUnpack<T, U> {
    x: Vec<T>,
    #[allow(dead_code)]
    y: Vec<U>,
}

#[derive(FromSuper)]
#[fromsuper(from_type = "BarGenMultiNoUnpack<#T,#U>")]
struct FooGenMultiNoUnpack<T> {
    x: Vec<T>,
}

#[test]
fn test_generics_multi_no_unpack() {
    let bar = BarGenMultiNoUnpack {
        x: vec!["abc"],
        y: vec![42],
    };

    let foo: FooGenMultiNoUnpack<_> = bar.into();
    assert_eq!(foo.x[0], "abc")
}

struct BarGenMulti<T, U> {
    x: Option<Vec<T>>,
    #[allow(dead_code)]
    y: Vec<U>,
}

#[derive(FromSuper)]
#[fromsuper(from_type = "BarGenMulti<#T,#U>", unpack = true)]
struct FooGenMulti<T> {
    x: Vec<T>,
}

#[test]
fn test_generics_multi() {
    let bar = BarGenMulti {
        x: Some(vec!["abc"]),
        y: vec![42],
    };

    let foo = FooGenMulti::try_from(bar).unwrap();
    assert_eq!(foo.x[0], "abc")
}

#[derive(PartialEq, Debug)]
struct BarRenameTest<T> {
    x: Option<T>,
    y: T,
}

#[derive(FromSuper, PartialEq, Debug)]
#[fromsuper(from_type = "BarRenameTest<T>", unpack = true)]
struct FooRenameTest1<T> {
    #[fromsuper(rename_from = "x")]
    z: T,
}

#[derive(FromSuper, PartialEq, Debug)]
#[fromsuper(from_type = "BarRenameTest<T>")]
struct FooRenameTest2<T> {
    #[fromsuper(rename_from = "y")]
    z: T,
}

#[test]
fn test_rename_from() {
    assert_eq!(
        FooRenameTest1 { z: 42 },
        BarRenameTest { x: Some(42), y: 53 }.try_into().unwrap()
    );

    assert_eq!(
        FooRenameTest2 { z: 53 },
        BarRenameTest { x: Some(42), y: 53 }.into()
    );
}

#[derive(Debug, Clone)]
struct BarGenericsMixed<T, U> {
    x: Vec<T>,
    y: Vec<U>,
}

#[derive(FromSuper)]
#[fromsuper(from_type = "BarGenericsMixed<#T,u32>")]
struct FooGenericsMixed<T> {
    x: Vec<T>,
}

#[derive(FromSuper)]
#[fromsuper(from_type = "BarGenericsMixed<#T,u32>")]
struct FooGenericsMixed2 {
    y: Vec<u32>,
}

#[test]
fn test_generics_mixed_free_and_specific() {
    let bar = BarGenericsMixed {
        x: vec!["huhu"],
        y: vec![42],
    };

    let foo: FooGenericsMixed<_> = bar.clone().into();
    assert_eq!(foo.x[0], "huhu");

    let foo: FooGenericsMixed2 = bar.into();
    assert_eq!(foo.y[0], 42);
}

#[derive(Debug, Clone)]
struct BarLifetime1<'a> {
    x: u32,
    y: &'a str,
}

#[derive(FromSuper)]
#[fromsuper(from_type = "BarLifetime1<'a>")]
struct FooLifetime1<'a> {
    y: &'a str,
}

#[derive(FromSuper)]
#[fromsuper(from_type = "BarLifetime1<'static>")]
struct FooLifetime2 {
    x: u32,
}

#[test]
fn test_lifetime() {
    let s = format!("Test {}", 123);
    let bar1 = BarLifetime1 { x: 42, y: &s[2..] };
    let bar2 = BarLifetime1 { x: 53, y: "hello" };

    let foo: FooLifetime1 = bar1.clone().into();
    assert_eq!(&s[2..], foo.y);

    let foo: FooLifetime2 = bar2.clone().into();
    assert_eq!(53, foo.x);
}

#[derive(Debug, Clone)]
struct BarComplex<'a, T: 'static, U, V, W> {
    a: u32,
    b: Option<&'a str>,
    c: Option<&'static T>,
    d: HashMap<U, V>,
    e: Option<(U, W)>,
}

#[derive(Debug, Clone)]
struct ComplexSub {
    x: String,
    y: Rc<u64>,
}

static COMPLEX_C: i16 = -42;

#[derive(FromSuper)]
#[fromsuper(from_type = "BarComplex<'a, #T, #U, #V, #W>")]
struct FooComplex1 {}

#[derive(FromSuper)]
#[fromsuper(from_type = "BarComplex<'a, #T, u8, char, #W>", unpack = "true")]
struct FooComplex2<'a, T: 'static> {
    b: &'a str,
    c: &'static T,
    #[fromsuper(no_unpack)]
    d: HashMap<u8, char>,
}

#[derive(FromSuper)]
#[fromsuper(
    from_type = "BarComplex<'a, #T, #U, char, ComplexSub>",
    unpack = "true"
)]
#[derive(Debug)]
struct FooComplex3<U> {
    #[fromsuper(no_unpack)]
    a: u32,
    #[fromsuper(no_unpack)]
    d: HashMap<U, char>,
    e: (U, ComplexSub),
}

#[test]
fn test_complex() {
    let s = format!("Test {}", 123);
    let bar = BarComplex {
        a: 42,
        b: Some(&s[2..]),
        c: Some(&COMPLEX_C),
        d: ([(1u8, 'a'), (2, 'b')]).into_iter().collect(),
        e: Some((
            16u8,
            ComplexSub {
                x: "hi there".to_string(),
                y: Rc::new(1_000_000_000_000),
            },
        )),
    };

    let _: FooComplex1 = bar.clone().into();

    let foo: FooComplex2<_> = bar.clone().try_into().unwrap();
    assert_eq!(foo.b, "st 123");
    assert_eq!(*foo.c, -42);
    assert_eq!(foo.d[&2], 'b');

    let foo: FooComplex3<_> = bar.clone().try_into().unwrap();
    assert_eq!(foo.a, 42);
    assert_eq!(foo.d[&2], 'b');
    assert_eq!(foo.e.0, 16);
    assert_eq!(foo.e.1.x, "hi there");
    assert_eq!(*foo.e.1.y, 1_000_000_000_000);
}
