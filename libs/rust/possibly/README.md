# `possibly`

A single exported macro called `possibly!()`, which works `std::matches!()` but allowing to return a value when the values matches.

It can be useful if you want to use pattern matching without dealing with extra cases.

The result is wrapped inside an `Option`.

## Usage

```rust
use possibly::possibly;

enum MyEnum {
    Foo(u32),
    Bar
}

let value = MyEnum::Foo(1);

// basic usage with simple match arm
assert_eq!(
    possibly!(value, MyEnum::Foo(b) => b), 
    Some(1)
);

// with match arm condition
assert_eq!(
    possibly!(value, MyEnum::Foo(b) if i > 5 => b),
    None
);
```