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
#[from_super(from_type = "BarGenMultiNoUnpack<T,U>")]
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
#[from_super(from_type = "BarGenMulti<T,U>", unpack = true)]
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
