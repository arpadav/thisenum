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

    // main2();
    main3();

    // // EnumConst example
    // assert_eq!(TestU8::Arm1.value(), &0x7F);
    // assert_eq!(TestU8::Arm1, 0x7F as u8);
    // assert_eq!(TestU8::Arm2.value(), &0x3B);

    // // EnumConst example 2
    // assert_eq!(TestStr::Arm1.value(), "this");
    // assert_eq!(TestStr::Arm1, "this");
    // assert_eq!(TestStr::Arm2.value(), "that");

    // // EnumConst example 3
    // assert_eq!(TestU8Slice4::Arm1.value(), b"\x7F\x7F\x7F\x7F\x67");
    // assert_eq!(TestU8Slice4::Arm2.value(), b"\x3B\x3B\x3B\x3B");
    // assert_eq!(TestU8Slice4::Arm1, b"\x7F\x7F\x7F\x7F\x67" as &[u8]);

    // // EnumConstAny example
    // assert!(TestStrAny::Arm1.value::<u8>().is_some());
    // let val = TestStrAny::Arm1.value::<u8>().unwrap();
    // println!("TestStrAny::Arm1.value() = {:?}", val);
    // println!("TestStrAny::Arm1.value() = {:?}", TestStrAny::Arm1.value::<u8>().unwrap());
    // let value = TestStrAny::Arm2.value::<Vec<f32>>();
    // println!("TestStrAny::Arm2.value() = {:?}", value);
    // assert!(TestStrAny::Arm2.value::<Vec<f32>>().is_none());
    // assert!(TestStrAny::Arm2.value::<&str>().is_some());
}

// #[derive(EnumConstAny, Debug)]
// enum MyEnum {
//     #[armtype(u8)]
//     #[value = 0xAA]
//     A,
//     #[value = "test3"]
//     B,
// }

// #[derive(EnumConstAny, Debug)]
// enum Tags {
//     #[value = b"\x00\x01"]
//     Key,
//     #[armtype(u16)]
//     #[value = 24250]
//     Length,
//     #[armtype(&[u8])]
//     #[value = b"\x00\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0a\x0b\x0c\x0d\x0e\x0f"]
//     Data,
// }

// fn main2() {
//     // [`EnumConstAny`] examples
//     assert!(MyEnum::A.value::<u8>().is_some());
//     assert!(MyEnum::A.value::<Vec<f32>>().is_none());
//     assert!(MyEnum::B.value::<u8>().is_none());
//     assert!(MyEnum::B.value::<&str>().is_some());

//     // An infered type. This will be as strict as possible,
//     // therefore [`&[u8]`] will fail but [`&[u8; 2]`] will succeed
//     assert!(Tags::Key.value::<&[u8; 2]>().is_some());
//     assert!(Tags::Key.value::<&[u8; 5]>().is_none());
//     assert!(Tags::Key.value::<&[u8]>().is_none());
//     assert!(u16::from_le_bytes(**Tags::Key.value::<&[u8; 2]>().unwrap()) == 0x0100);

//     // casting as anything other than the defined / inferred type will
//     // fail, since this uses [`downcast_ref`] from [`std::any::Any`]
//     assert!(Tags::Length.value::<u16>().is_some());
//     assert!(Tags::Length.value::<u32>().is_none());
//     assert!(Tags::Length.value::<u64>().is_none());

//     // however, can always convert to a different type
//     // after value is successfully acquired
//     assert!(*Tags::Length.value::<u16>().unwrap() as u32 == 24250);
// }

// use enum_const::EnumConst;

#[derive(EnumConst, Debug)]
#[armtype(i32)]
enum MyEnum {
    #[value = 0]
    A,
    #[value = 1]
    B,
}

#[derive(EnumConst, Debug)]
#[armtype(&[u8])]
enum Tags {
    #[value = b"\x00\x01\x7f"]
    Key,
    #[value = b"\xba\x5e"]
    Length,
    #[value = b"\x00\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0a\x0b\x0c\x0d\x0e\x0f"]
    Data,
}

fn main3() {
    // it's prefered to use the function call to `value` 
    // to get a [`&'static T`] reference to the value
    assert_eq!(MyEnum::A.value(), &0);
    assert_eq!(MyEnum::B.value(), &1);
    assert_eq!(Tags::Key.value(), b"\x00\x01\x7f");
    assert_eq!(Tags::Length.value(), b"\xba\x5e");
    assert_eq!(Tags::Data.value(), b"\x00\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0a\x0b\x0c\x0d\x0e\x0f");

    // can also check equality without the function call. This must compare the input 
    // type defined in `#[armtype = ...]`
    assert_eq!(Tags::Length, b"\xba\x5e");
}