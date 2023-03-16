use fromsuper::FromSuper;

use ::anyhow;

struct Bar {
    x: Option<Vec<u32>>,
}

#[derive(FromSuper)]
#[from_super(from_type = "Bar")]
struct Foo {
    #[allow(dead_code)]
    x: Vec<u32>,
}

fn anyhow_convert_inner_1() -> anyhow::Result<Foo> {
    let foo = Foo::from_super_try_unwrap(Bar { x: Some(vec![42]) })?;
    Ok(foo)
}

fn anyhow_convert_inner_2() -> anyhow::Result<Foo> {
    let foo = Foo::from_super_try_unwrap(Bar { x: None })?;
    Ok(foo)
}

#[test]
fn anyhow_convert() {
    assert!(anyhow_convert_inner_1().is_ok());
    assert!(anyhow_convert_inner_2().is_err());
}
