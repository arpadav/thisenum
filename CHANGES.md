# 0.2.0

* Automatically derive `Debug`: `format!("MyEnum::Arm1: {}", &self.value())`
* Optionally derive `PartialEq` using the `eq` feature (e.g.: `thisenum = { version = "0.2.0", features = ["eq"] }`)
* New errors, when value's are not unique. Previously this would expand into:

```rust
match value {
    1 => Ok(MyEnum::Arm1),
    2 => Ok(MyEnum::Arm2),
    3 => Ok(MyEnum::Arm3),
    1 => Ok(MyEnum::ArmX), // unreachable pattern
    _ => Err(())
}
```

Now expands to:

```rust
match value {
    2 => Ok(MyEnum::Arm2),
    3 => Ok(MyEnum::Arm3),
    // Multiple associated enum arms defined with value `{0}`
    1 => Err(thisenum::Error::UnreachableValue(value)),
    // Unable to convert `{0}` to `{1}`
    _ => Err(thisenum::Error::InvalidValue(value, "MyEnum")),
}
```

* New errors, when enum variant is unable to be returned due to nested args. Previously this would not expand. Now it expands to:

```rust
#[derive(Const)]
#[armtype(u8)]
enum MyOtherEnum {
    #[value = 1]
    Arm1(u8, u8),
    #[value = 2]
    Arm2,
}

match value {
    1 => Err(thisenum::Error::UnableToReturnVariant("Arm1")),
    2 => Ok(MyOtherEnum::Arm2),
    _ => Err(thisenum::Error::InvalidValue(value, "MyOtherEnum")),
}
```
