## enum-const

Assign constant literals to enum arms! What fun!

```rust
use enum_const::EnumConst;

#[derive(EnumConst)
#[armtype(&[u8])]
/// https://exiftool.org/TagNames/EXIF.html
enum ExifTag {
    #[value = b"\x01\x00"]
    ImageWidth,
    #[value = b"\x01\x01"]
    ImageHeight,
    #[value = b"\x01\x02"]
    BitsPerSamole,
    #[value = b"\x01\x03"]
    Compression,
}

assert_eq!(ExifTag::ImageWidth.value(), b"\x01\x00")
assert_eq!(ExifTag::ImageWidth, b"\x01\x00")
```