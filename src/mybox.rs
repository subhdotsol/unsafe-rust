// // owing a pointer to heap allocated data
// // - memory allocated when box created dropped when box is out of scope

// // Deref
// // DerefMut
// // Drop

// use std::{
//     alloc::{Layout, alloc, dealloc},
//     ops::{Deref, DerefMut},
//     ptr::NonNull,
// };

// pub struct MyBox<T> {
//     ptr: NonNull<T>,
// }

// impl<T> MyBox<T> {
//     pub fn new(value: T) -> Self {
//         unsafe {
//             let layout = Layout::new::<T>();
//             let raw_ptr = alloc(layout) as *mut T; // Option<NonNull<T>>

//             if raw_ptr.is_null() {
//                 std::alloc::handle_alloc_error(layout);
//             }

//             raw_ptr.write(value);

//             MyBox {
//                 ptr: NonNull::new_unchecked(raw_ptr),
//             }
//         }
//     }
// }

// impl<T> Deref for MyBox<T> {
//     type Target = T;
//     fn deref(&self) -> &T {
//         unsafe { self.ptr.as_ref() }
//     }
// }

// impl<T> DerefMut for MyBox<T> {
//     fn deref_mut(&mut self) -> &mut T {
//         unsafe { self.ptr.as_mut() }
//     }
// }

// impl<T> Drop for MyBox<T> {
//     fn drop(&mut self) {
//         unsafe {
//             let layout = Layout::new::<T>();

//             // Drop the value first
//             std::ptr::drop_in_place(self.ptr.as_ptr());

//             // Free the memory
//             dealloc(self.ptr.as_ptr() as *mut u8, layout);
//         }
//     }
// }

// #[cfg(test)]
// mod tests {
//     use std::cell::Cell;

//     use super::*;

//     #[test]
//     fn test_basic_deref() {
//         let mut x = MyBox::new(42);
//         assert_eq!(*x, 42);
//     }

//     #[test]
//     fn test_mut_deref() {
//         let mut x = MyBox::new(10);
//         *x = 99;
//         assert_eq!(*x, 99);
//     }

//     #[test]
//     fn test_string() {
//         let x = MyBox::new(String::from("naina"));
//         assert_eq!(x.len(), 5); // deref coercion works
//     }

//     #[test]
//     fn test_drop_run() {
//         struct TestDrop<'a> {
//             flag: &'a Cell<bool>,
//         }

//         impl<'a> Drop for TestDrop<'a> {
//             fn drop(&mut self) {
//                 self.flag.set(true);
//             }
//         }

//         let flag = Cell::new(false);
//         {
//             let _x = MyBox::new(TestDrop { flag: &flag });
//             assert_eq!(flag.get(), false);
//         }
//         // after scope -> Drop should run
//         assert_eq!(flag.get(), true);
//     }
// }

use std::{
    // Layout describes how much memory we need and its alignment.
    alloc::{Layout, alloc, dealloc},

    // Deref allows *x and automatic dereferencing.
    // DerefMut allows modifying the value through *x.
    ops::{Deref, DerefMut},

    // NonNull<T> is a raw pointer that is guaranteed to be non-null.
    ptr::NonNull,
};

// Our own smart pointer type.
//
// MyBox<T> owns a T that lives somewhere on the heap.
//
// Instead of storing T directly:
//
//     struct MyBox<T> {
//         value: T
//     }
//
// we store a pointer to T:
//
//     struct MyBox<T> {
//         ptr: NonNull<T>
//     }
pub struct MyBox<T> {
    ptr: NonNull<T>,
}

impl<T> MyBox<T> {
    // Create a new MyBox containing `value`.
    //
    // Example:
    //
    // let x = MyBox::new(42);
    //
    // Conceptually:
    //
    // Stack:
    //     x
    //     |
    //     v
    //
    // Heap:
    //     [42]
    pub fn new(value: T) -> Self {
        // We are going to manually work with raw pointers
        // and manually allocated memory.
        //
        // Rust normally protects us from this, so we need
        // an unsafe block.
        unsafe {
            // Create a Layout describing how memory for T
            // should look.
            //
            // For example, if T = i32:
            //
            // size      = 4 bytes
            // alignment = 4 bytes
            let layout = Layout::new::<T>();

            // Allocate raw memory on the heap.
            //
            // `alloc(layout)` returns:
            //
            //     *mut u8
            //
            // which is a raw pointer to bytes.
            //
            // We cast it to:
            //
            //     *mut T
            //
            // because this memory will contain a T.
            let raw_ptr = alloc(layout) as *mut T;

            // `alloc()` can return a null pointer if allocation
            // fails.
            //
            // We cannot safely continue with a null pointer.
            if raw_ptr.is_null() {
                // Tell Rust that allocation failed.
                //
                // This normally terminates the process by
                // invoking the allocation error handler.
                std::alloc::handle_alloc_error(layout);
            }

            // Put `value` into the allocated memory.
            //
            // IMPORTANT:
            //
            // `raw_ptr` points to uninitialized memory.
            //
            // `write()` initializes that memory with `value`.
            //
            // If value = 42:
            //
            // Before:
            //     [uninitialized memory]
            //
            // After:
            //     [42]
            raw_ptr.write(value);

            // Construct our MyBox.
            //
            // `NonNull::new_unchecked()` converts the raw pointer
            // into a NonNull<T>.
            //
            // We are allowed to use `new_unchecked()` because
            // we already checked above that raw_ptr is not null.
            MyBox {
                ptr: NonNull::new_unchecked(raw_ptr),
            }
        }
    }
}

impl<T> Deref for MyBox<T> {
    // Deref says:
    //
    // "When somebody dereferences MyBox<T>, treat it as T."
    //
    // So:
    //
    //     *x
    //
    // can access the T stored inside MyBox<T>.
    type Target = T;

    // This function is called when Rust needs to dereference
    // MyBox<T> as an immutable reference.
    fn deref(&self) -> &T {
        unsafe {
            // `self.ptr` is NonNull<T>.
            //
            // `as_ref()` converts it into:
            //
            //     &T
            //
            // In other words:
            //
            // raw pointer -> safe shared reference
            self.ptr.as_ref()
        }
    }
}

impl<T> DerefMut for MyBox<T> {
    // This is the mutable version of deref.
    //
    // It allows:
    //
    //     *x = 99;
    //
    // when x is a mutable MyBox.
    fn deref_mut(&mut self) -> &mut T {
        unsafe {
            // Convert the NonNull<T> pointer into:
            //
            //     &mut T
            //
            // This gives mutable access to the value stored
            // on the heap.
            self.ptr.as_mut()
        }
    }
}

impl<T> Drop for MyBox<T> {
    // Rust automatically calls this when a MyBox<T>
    // goes out of scope.
    //
    // For example:
    //
    // {
    //     let x = MyBox::new(42);
    // } // <-- drop() runs here
    fn drop(&mut self) {
        unsafe {
            // Recreate the same Layout that we used
            // when allocating the memory.
            //
            // IMPORTANT:
            //
            // The layout used for deallocation must match
            // the layout used for allocation.
            let layout = Layout::new::<T>();

            // First destroy the T stored in the allocated memory.
            //
            // `drop_in_place()` runs T's destructor.
            //
            // This is particularly important for types such as:
            //
            //     String
            //     Vec<T>
            //     File
            //     custom types implementing Drop
            //
            // For i32, there's essentially nothing to clean up.
            //
            // For String, its internal heap allocation needs
            // to be released.
            std::ptr::drop_in_place(self.ptr.as_ptr());

            // Now release the heap memory itself.
            //
            // `dealloc()` expects a *mut u8, so we cast the
            // *mut T pointer back to *mut u8.
            //
            // Notice the order:
            //
            //     1. Drop T
            //     2. Deallocate memory
            //
            // NOT:
            //
            //     1. Deallocate memory
            //     2. Drop T
            //
            // because after deallocation the memory is no longer
            // valid to access.
            dealloc(self.ptr.as_ptr() as *mut u8, layout);
        }
    }
}

// Only compile this module when running tests.
#[cfg(test)]
mod tests {

    // Cell allows us to modify a value through a shared reference.
    //
    // We use it to verify that Drop actually happened.
    use std::cell::Cell;

    // Bring everything from the parent module into scope.
    //
    // This lets the tests use MyBox directly.
    use super::*;

    // Mark this function as a test.
    #[test]
    fn test_basic_deref() {
        // Create a MyBox containing 42.
        //
        // `mut` isn't actually needed in this particular test.
        let mut x = MyBox::new(42);

        // `*x` works because we implemented Deref.
        //
        // Conceptually:
        //
        //     x
        //     ↓
        //     MyBox
        //     ↓
        //     pointer
        //     ↓
        //     42
        //
        // assert_eq! checks that both values are equal.
        assert_eq!(*x, 42);
    }

    #[test]
    fn test_mut_deref() {
        // Create MyBox containing 10.
        let mut x = MyBox::new(10);

        // Because x is mutable and we implemented DerefMut,
        // Rust allows us to write:
        //
        //     *x = 99;
        //
        // This changes the T stored on the heap.
        *x = 99;

        // Read the value back.
        assert_eq!(*x, 99);
    }

    #[test]
    fn test_string() {
        // Store a String inside MyBox.
        //
        // The String itself is stored inside the memory
        // allocated by MyBox.
        //
        // The String also internally owns another heap
        // allocation containing "naina".
        let x = MyBox::new(String::from("naina"));

        // `x.len()` works even though MyBox doesn't have
        // a `len()` method.
        //
        // Rust uses Deref to reach String:
        //
        //     x
        //     ↓
        //     MyBox<String>
        //     ↓ Deref
        //     String
        //     ↓
        //     len()
        //
        // This is called deref coercion/deref behavior.
        assert_eq!(x.len(), 5);
    }

    #[test]
    fn test_drop_run() {
        // A small type used to check whether Drop runs.
        //
        // It contains a reference to a Cell<bool>.
        struct TestDrop<'a> {
            flag: &'a Cell<bool>,
        }

        // Give TestDrop its own destructor.
        impl<'a> Drop for TestDrop<'a> {
            // This function runs when TestDrop is destroyed.
            fn drop(&mut self) {
                // Change the flag from false to true.
                self.flag.set(true);
            }
        }

        // Initially the flag is false.
        let flag = Cell::new(false);

        // Start a new scope.
        {
            // Create a MyBox containing TestDrop.
            //
            // At this point TestDrop has NOT been dropped yet.
            let _x = MyBox::new(TestDrop { flag: &flag });

            // Therefore the flag should still be false.
            assert_eq!(flag.get(), false);
        }

        // `_x` went out of scope above.
        //
        // Therefore:
        //
        // MyBox::drop()
        //      ↓
        // drop_in_place(TestDrop)
        //      ↓
        // TestDrop::drop()
        //      ↓
        // flag becomes true
        //
        assert_eq!(flag.get(), true);
    }
}
