use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::Value;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
enum EnumUnit {
    X,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
enum EnumNewType {
    N(u64),
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
enum EnumTuple {
    T(u64, String),
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
enum EnumStruct {
    S { x: u64 },
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
enum EnumStructUntagged {
    #[serde(untagged)]
    U { x: u64, y: u64 },
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct StructNewType(bool);

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct StructTuple(u64, String);

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TestStruct {
    u_64: u64,
    u_32: u32,
    u_16: u16,
    u_8: u8,
    i_64: i64,
    i_32: i32,
    i_16: i16,
    i_8: i8,
    flag: bool,
    string: String,
    map: HashMap<String, String>,
    arr: Vec<String>,
    value: Value,
    unit: (),
    enum_unit: EnumUnit,
    enum_tuple: EnumTuple,
    enum_struct: EnumStruct,
    enum_new_type: EnumNewType,
    enum_untagged: EnumStructUntagged,
    struct_tuple: StructTuple,
    struct_new_type: StructNewType,
    tuple: (String, bool),
    optional_string: Option<String>,
}

pub(crate) fn test_value() -> Value {
    Value::object([
        ("u64", Value::unsigned(u64::MAX)),
        ("u32", u32::MAX.into()),
        ("u16", u16::MAX.into()),
        ("u8", u8::MAX.into()),
        ("i64", i64::MAX.into()),
        ("i32", i32::MAX.into()),
        ("i16", i16::MAX.into()),
        ("i8", i8::MAX.into()),
        ("flag", Value::bool(true)),
        ("string", Value::string("foo")),
        ("map", Value::object([("foo", Value::string("bar"))])),
        (
            "arr",
            Value::array([
                Value::string("one"),
                Value::string("two"),
                Value::string("three"),
            ]),
        ),
        ("value", Value::bool(true)),
        ("unit", Value::Null),
        ("enumUnit", Value::string("X")),
        (
            "enumTuple",
            Value::object([(
                "T",
                Value::array([Value::unsigned(1000), Value::string("foo")]),
            )]),
        ),
        (
            "enumStruct",
            Value::object([("S", Value::object([("x", Value::unsigned(1000))]))]),
        ),
        ("enumNewType", Value::object([("N", Value::unsigned(1000))])),
        (
            "enumUntagged",
            Value::object([("x", Value::unsigned(1000)), ("y", Value::unsigned(1000))]),
        ),
        (
            "structTuple",
            Value::array([Value::unsigned(1000), Value::string("foo")]),
        ),
        ("structNewType", Value::bool(true)),
        (
            "tuple",
            Value::array([Value::string("foo"), Value::bool(true)]),
        ),
        ("optionalString", Value::string("foo")),
    ])
}

pub(crate) fn test_struct() -> TestStruct {
    TestStruct {
        u_64: u64::MAX,
        u_32: u32::MAX,
        u_16: u16::MAX,
        u_8: u8::MAX,
        i_64: i64::MAX,
        i_32: i32::MAX,
        i_16: i16::MAX,
        i_8: i8::MAX,
        flag: true,
        string: "foo".into(),
        map: HashMap::from([("foo".into(), "bar".into())]),
        arr: vec!["one".into(), "two".into(), "three".into()],
        value: Value::bool(true),
        unit: (),
        enum_unit: EnumUnit::X,
        enum_tuple: EnumTuple::T(1000, "foo".into()),
        enum_struct: EnumStruct::S { x: 1000 },
        enum_new_type: EnumNewType::N(1000),
        enum_untagged: EnumStructUntagged::U { x: 1000, y: 1000 },
        struct_tuple: StructTuple(1000, "foo".into()),
        struct_new_type: StructNewType(true),
        tuple: ("foo".into(), true),
        optional_string: Some("foo".into()),
    }
}
