use fromsuper::FromSuper;

#[derive(Clone)]
struct Bar {
    bar: Option<u32>,
    baz: Option<Option<String>>,
}

#[derive(PartialEq, Eq, Debug, FromSuper)]
#[from_super(from_type = "crate::Bar", unpack = true)]
struct Foo {
    bar: u32,
    #[from_super(no_unpack = false)]
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
#[from_super(from_type = "BarGen<T>", unpack = true)]
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
#[from_super(from_type = "BarGenMultiNoUnpack<#T,#U>")]
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
#[from_super(from_type = "BarGenMulti<#T,#U>", unpack = true)]
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
#[from_super(from_type = "BarRenameTest<T>", unpack = true)]
struct FooRenameTest1<T> {
    #[from_super(rename_from = "x")]
    z: T,
}

#[derive(FromSuper, PartialEq, Debug)]
#[from_super(from_type = "BarRenameTest<T>")]
struct FooRenameTest2<T> {
    #[from_super(rename_from = "y")]
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
#[from_super(from_type = "BarGenericsMixed<#T,u32>")]
struct FooGenericsMixed<T> {
    x: Vec<T>,
}

#[derive(FromSuper)]
#[from_super(from_type = "BarGenericsMixed<#T,u32>")]
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
#[from_super(from_type = "BarLifetime1<'a>")]
struct FooLifetime1<'a> {
    y: &'a str,
}

#[derive(FromSuper)]
#[from_super(from_type = "BarLifetime1<'static>")]
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
