// owing a pointer to heap allocated data
// - memory allocated when box created dropped when box is out of scope

// Deref
// DerefMut
// Drop

use std::{
    alloc::{Layout, alloc, dealloc},
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

pub struct MyBox<T> {
    ptr: NonNull<T>,
}

impl<T> MyBox<T> {
    pub fn new(value: T) -> Self {
        unsafe {
            let layout = Layout::new::<T>();
            let raw_ptr = alloc(layout) as *mut T; // Option<NonNull<T>>

            if raw_ptr.is_null() {
                std::alloc::handle_alloc_error(layout);
            }

            raw_ptr.write(value);

            MyBox {
                ptr: NonNull::new_unchecked(raw_ptr),
            }
        }
    }
}

impl<T> Deref for MyBox<T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { self.ptr.as_ref() }
    }
}

impl<T> DerefMut for MyBox<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { self.ptr.as_mut() }
    }
}

impl<T> Drop for MyBox<T> {
    fn drop(&mut self) {
        unsafe {
            let layout = Layout::new::<T>();

            // Drop the value first
            std::ptr::drop_in_place(self.ptr.as_ptr());

            // Free the memory
            dealloc(self.ptr.as_ptr() as *mut u8, layout);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use super::*;

    #[test]
    fn test_basic_deref() {
        let mut x = MyBox::new(42);
        assert_eq!(*x, 42);
    }

    #[test]
    fn test_mut_deref() {
        let mut x = MyBox::new(10);
        *x = 99;
        assert_eq!(*x, 99);
    }

    #[test]
    fn test_string() {
        let x = MyBox::new(String::from("naina"));
        assert_eq!(x.len(), 5); // deref coercion works
    }

    #[test]
    fn test_drop_run() {
        struct TestDrop<'a> {
            flag: &'a Cell<bool>,
        }

        impl<'a> Drop for TestDrop<'a> {
            fn drop(&mut self) {
                self.flag.set(true);
            }
        }

        let flag = Cell::new(false);
        {
            let _x = MyBox::new(TestDrop { flag: &flag });
            assert_eq!(flag.get(), false);
        }
        // after scope -> Drop should run
        assert_eq!(flag.get(), true);
    }
}

