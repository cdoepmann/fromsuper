# Fromsuper

`fromsuper` provides a procedural macro that helps with converting (large) super structs to (smaller) sub structs, by discarding unneeded data. It can also automatically unpack `Option`s during this conversion. It implements `TryFrom` if `Option`s need to be unpacked, and `From` otherwise.

It can be useful e.g., when working with large parser outputs of which only a subset is actually needed. Reducing such structs to the data actually needed improves maintainability. If the original struct contains lots of `Option`s, unpacking them validates that the needed data is present and greatly improves ergonomics of further handling.

## Basic Usage

Include `fromsuper` in your project by adding the following to your `Cargo.toml`:

```toml
fromsuper = "0.1"
```

You may also want to `use` the derive macro:

```rust
use fromsuper::FromSuper;
```

Options for the derive macro are specified by using the `fromsuper` attribute. The only option that is necessary is `from_type`, defining the super struct to convert from:

```rust
struct Bar {
    a: u32,
    b: String,
    c: HashSet<u64>,
    d: ComplexData,
}

#[derive(FromSuper)]
#[fromsuper(from_type = "Bar")]
struct Foo {
    a: u32,
    c: HashSet<u64>,
}

let bar = Bar { ... }
let foo: Foo = bar.into() // using Foo's derived implementation of From<Bar>
```

If a sub struct's field is not named the same as the original one, the field attribute `rename_from` can be used to specify the mapping:

```rust
#[derive(FromSuper)]
#[fromsuper(from_type = "Bar")]
struct Foo {
    a: u32,
    #[fromsuper(rename_from = "c")]
    new_name: HashSet<u64>,
}
```

## Unpacking `Option`s

The automatic unpacking of `Option`s from the original struct can be enabled by adding the `unpack` argument. Single fields can opt-out of the unpacking (`unpack = false`), e.g., when not all original fields are `Option`s, or when you can tolerate `None` values. When unpacking is enabled, `TryFrom` is implemented instead of `From`, in order to fail when required values are `None`:

```rust
struct Bar {
    a: Option<u32>,
    b: String,
    c: Option<HashSet<u64>>,
    d: Option<ComplexData>,
}

#[derive(FromSuper)]
#[fromsuper(from_type = "Bar", unpack = true)]
struct Foo {
    #[fromsuper(unpack = false)]
    b: String,
    c: HashSet<u64>,
    d: ComplexData,
}

let bar = Bar { ... }
let foo: Foo = bar.try_into()? // using Foo's derived implementation of TryFrom<Bar>
```

## Generics

`derive(FromSuper)` can handle many situations in which generics are involved. Both, the super and the sub struct, can have type parameters. When specifying the super struct, however, it is impossible to decide whether e.g. the `T` in `Bar<T>` is a  type parameter or a concrete type. In order to differentiate the two meanings, generic type parameters have to be prefixed with a `#` sign:

```rust
struct Bar<T, U> {
    x: Vec<T>,
    y: Vec<U>,
    z: U,
}

#[derive(FromSuper)]
#[fromsuper(from_type = "Bar<#T,u32>")]
struct FooGenericsMixed<T> {
    x: Vec<T>,
    z: u32,
}
```

This way, it is possible to reduce the number of type parameters for the sub struct, if its fields do not require them.

## Contributions

Since it is hard to predict all possible usage scenarios of the proc macro, there may be situations that are not properly handled. Please let me know by filing an issue.

## License

This project is licensed under the terms of the MIT License, as well as the Apache 2.0 License. You are free to choose whichever suits your needs best.
