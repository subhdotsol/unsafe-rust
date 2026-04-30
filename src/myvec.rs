// implement own vec
// and then new , push , pop , grow , drop

use ::std::ptr;
use std::alloc::{Layout, alloc, realloc};

pub struct MyVec<T> {
    ptr: *mut T,
    len: usize,
    cap: usize,
}

impl<T> MyVec<T> {
    pub fn new() -> Self {
        Self {
            ptr: ptr::null_mut(),
            len: 0,
            cap: 0,
        }
    }

    pub fn grow(&mut self) {
        let new_cap = if self.cap == 0 { 1 } else { self.cap * 2 };

        let new_layout = Layout::array::<T>(new_cap).unwrap();

        unsafe {
            let new_ptr = if self.cap == 0 {
                alloc(new_layout)
            } else {
                let old_layout = Layout::array::<T>(self.cap).unwrap();
                realloc(self.ptr as *mut u8, old_layout, new_layout.size())
            };
            self.ptr = new_ptr as *mut T;
        }
        self.cap = new_cap;
    }

    pub fn push(&mut self, value: T) {
        if self.len == self.cap {
            self.grow();
        }
        unsafe {
            ptr::write(self.ptr.add(self.len), value);
        }
        self.len += 1;
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }
        self.len -= 1;
        unsafe { Some(ptr::read(self.ptr.add(self.len))) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let v: MyVec<i32> = MyVec::new();
        assert_eq!(v.len, 0);
        assert_eq!(v.cap, 0);
        assert!(v.ptr.is_null());
    }

    #[test]
    fn test_push_and_pop() {
        let mut v = MyVec::new();

        v.push(10);
        v.push(20);

        assert_eq!(v.len, 2);
        assert_eq!(v.pop(), Some(20));
        assert_eq!(v.pop(), Some(10));
        assert_eq!(v.pop(), None);
    }

    #[test]
    fn test_grow() {
        let mut v = MyVec::new();

        for i in 0..100 {
            v.push(i);
        }

        assert_eq!(v.len, 100);

        for i in (0..100).rev() {
            assert_eq!(v.pop(), Some(i));
        }
    }
}
