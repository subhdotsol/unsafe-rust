// owing a pointer to heap allocated data
// - memory allocated when box created dropped when box is out of scope

// Deref
// DerefMut
// Drop

use std::{
    alloc::{Layout, alloc},
    ops::Deref,
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
