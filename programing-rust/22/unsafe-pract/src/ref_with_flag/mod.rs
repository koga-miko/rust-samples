use std::marker::PhantomData;
use std::mem::align_of;

pub struct RefWithFlag<'a, T: 'a> {
    ptr_and_bit:usize,
    behaves_like: PhantomData<&'a T>
}

impl<'a, T: 'a> RefWithFlag<'a, T> {
    pub fn new(ptr: &'a T, flag: bool) -> Self {
        assert!(align_of::<T>() % 2 == 0);
        Self {
            ptr_and_bit: ptr as *const T as usize | flag as usize,
            behaves_like: PhantomData
        }
    }

    pub fn get_ref(&self) -> &'a T {
        unsafe {
            let ptr = self.ptr_and_bit & !1;
            &*(ptr as *const T)
        }
    }

    pub fn get_flag(&self) -> bool {
        (self.ptr_and_bit & 1)  != 0 
    }
}
