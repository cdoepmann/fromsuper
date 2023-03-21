use fromsuper::FromSuper;

use ::anyhow;

struct Bar {
    x: Option<Vec<u32>>,
}

#[derive(FromSuper)]
#[fromsuper(from_type = "Bar", unpack = true)]
struct Foo {
    #[allow(dead_code)]
    x: Vec<u32>,
}

fn anyhow_convert_inner_1() -> anyhow::Result<Foo> {
    let foo = Foo::try_from(Bar { x: Some(vec![42]) })?;
    Ok(foo)
}

fn anyhow_convert_inner_2() -> anyhow::Result<Foo> {
    let foo = Foo::try_from(Bar { x: None })?;
    Ok(foo)
}

#[test]
fn anyhow_convert() {
    assert!(anyhow_convert_inner_1().is_ok());
    assert!(anyhow_convert_inner_2().is_err());
}
