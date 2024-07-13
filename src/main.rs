use enum_const::{
    EnumConst,
    EnumConstAny,
};

#[derive(Debug, EnumConst)]
#[armtype(u8)]
enum TestU8 {
    #[value = 0x7f]
    Arm1,
    #[value = 0x3B]
    Arm2,
}

#[derive(Debug, EnumConst)]
#[armtype(&str)]
enum TestStr {
    #[value = "this"]
    Arm1,
    #[value = "that"]
    Arm2,
}

#[derive(Debug, EnumConst)]
#[armtype(&[u8])]
enum TestU8Slice4 {
    #[value = b"\x7F\x7F\x7F\x7F\x67"]
    Arm1,
    #[value = b"\x3B\x3B\x3B\x3B"]
    Arm2,
}

// #[derive(EnumConst)]
// #[arm_type(Vec<usize>)]
// enum TestVecu8 {
//     #[value = vec![1, 2, 3]]
//     Arm1,
//     #[value = vec![4, 5, 6]]
//     Arm2,
// }

#[derive(Debug, EnumConstAny)]
enum TestStrAny {
    #[armtype(u8)]
    #[value = 0xAA]
    Arm1,
    #[value = "test3"]
    Arm2,
}

fn main() {
    // EnumConst example
    assert_eq!(TestU8::Arm1.value(), &0x7F);
    assert_eq!(TestU8::Arm1, 0x7F as u8);
    assert_eq!(TestU8::Arm2.value(), &0x3B);

    // EnumConst example 2
    assert_eq!(TestStr::Arm1.value(), "this");
    assert_eq!(TestStr::Arm1, "this");
    assert_eq!(TestStr::Arm2.value(), "that");

    // EnumConst example 3
    assert_eq!(TestU8Slice4::Arm1.value(), b"\x7F\x7F\x7F\x7F\x67");
    assert_eq!(TestU8Slice4::Arm2.value(), b"\x3B\x3B\x3B\x3B");
    assert_eq!(TestU8Slice4::Arm1, b"\x7F\x7F\x7F\x7F\x67" as &[u8]);

    // EnumConstAny example
    assert!(TestStrAny::Arm1.value::<u8>().is_some());
    let val = TestStrAny::Arm1.value::<u8>().unwrap();
    println!("TestStrAny::Arm1.value() = {:?}", val);
    println!("TestStrAny::Arm1.value() = {:?}", TestStrAny::Arm1.value::<u8>().unwrap());
    let value = TestStrAny::Arm2.value::<Vec<f32>>();
    println!("TestStrAny::Arm2.value() = {:?}", value);
    assert!(TestStrAny::Arm2.value::<Vec<f32>>().is_none());
    assert!(TestStrAny::Arm2.value::<&str>().is_some());
}