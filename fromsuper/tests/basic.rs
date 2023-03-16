use fromsuper::FromSuper;

#[derive(Clone)]
struct Bar {
    bar: Option<u32>,
    baz: Option<Option<String>>,
}

#[derive(PartialEq, Eq, Debug, FromSuper)]
#[from_super(from_type = "crate::Bar")]
struct Foo {
    bar: u32,
    #[from_super(no_unwrap = false)]
    baz: Option<String>,
}

#[test]
fn basic_unwrap() {
    assert_eq!(
        Foo { bar: 42, baz: None },
        Foo::from_super_try_unwrap(Bar {
            bar: Some(42),
            baz: Some(None),
        })
        .unwrap()
    );

    assert!(Foo::from_super_try_unwrap(Bar {
        bar: Some(42),
        baz: None,
    })
    .is_err());
}

#[test]
fn basic_try_unwrap() {
    assert!(Foo::from_super_try_unwrap(Bar {
        bar: Some(42),
        baz: None,
    })
    .is_err());

    assert!(Foo::from_super_try_unwrap(Bar {
        bar: Some(42),
        baz: Some(None),
    })
    .is_ok());
}

struct BarGen<T> {
    x: Option<Vec<T>>,
}

#[derive(FromSuper)]
#[from_super(from_type = "BarGen<T>")]
struct FooGen<T> {
    x: Vec<T>,
}

#[test]
fn test_generics_single() {
    let bar = BarGen {
        x: Some(vec!["abc"]),
    };

    let foo = FooGen::from_super_unwrap(bar);
    assert_eq!(foo.x[0], "abc")
}

struct BarGenMulti<T, U> {
    x: Option<Vec<T>>,
    y: Vec<U>,
}

#[derive(FromSuper)]
#[from_super(from_type = "BarGenMulti<T,U>")]
struct FooGenMulti<T, U> {
    x: Vec<T>,
}

#[test]
fn test_generics_multi() {
    let bar = BarGenMulti {
        x: Some(vec!["abc"]),
        y: vec![42],
    };

    let foo = FooGenMulti::from_super_unwrap(bar);
    assert_eq!(foo.x[0], "abc")
}
