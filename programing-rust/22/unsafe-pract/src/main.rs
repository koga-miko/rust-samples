mod ref_with_flag;
mod gap_buffer;
use ref_with_flag::RefWithFlag;
use std::mem::size_of;
use gap_buffer::GapBuffer;

fn main() {
//    let a = my_ascii::Ascii::from_bytes(Vec::from(b"Hello, \xcc\xcd\xce\xcfworld!")).unwrap();
    let a = my_ascii::Ascii::from_bytes(Vec::from(b"Hello, valid fworld!")).unwrap();
    let str: String = a.into();
    println!("{}", str);

    let i = 10;
    trustworthyy(&i);
    println!("{}", i);

    let ref_with_flag = RefWithFlag::new(&i, true);
    println!("Size of RefWithFlag: {}", size_of::<RefWithFlag<isize>>());
    println!("ref: {}, flag: {}", ref_with_flag.get_ref(), ref_with_flag.get_flag());

    let mut gap_buffer = GapBuffer::<char>::new();
    gap_buffer.set_position(0);
    gap_buffer.print();
    gap_buffer.insert('a');
    gap_buffer.print();
    gap_buffer.insert('b');
    gap_buffer.print();
    gap_buffer.insert('c');
    gap_buffer.print();
    gap_buffer.insert('d');
    gap_buffer.print();
    gap_buffer.insert('e');
    gap_buffer.print();
    gap_buffer.set_position(3);
    gap_buffer.print();
    gap_buffer.insert('X');
    gap_buffer.print();
    gap_buffer.remove();
    gap_buffer.print();
}

fn trustworthyy(shared: &i32) {
    #[allow(invalid_reference_casting)]
    unsafe {
        let mutable = shared as *const i32 as *mut i32;
        *mutable = 20;
    }
}

mod my_ascii {
    #[derive(Debug, Eq, PartialEq)]
    pub struct Ascii(
        // 0..=127
        Vec<u8>
    );
    impl Ascii {
        pub fn from_bytes(bytes: Vec<u8>) -> Result<Ascii, NotAsciiError> {
            if bytes.iter().any(|&byte| !byte.is_ascii()) {
                return Err(NotAsciiError(bytes));
            }
            Ok(Ascii(bytes))
        }
    }

    #[derive(Debug, Eq, PartialEq)]
    pub struct NotAsciiError(pub Vec<u8>);

    impl From<Ascii> for String {
        fn from(ascii: Ascii) -> Self {
            unsafe { String::from_utf8_unchecked(ascii.0) }
        }
    }
}
