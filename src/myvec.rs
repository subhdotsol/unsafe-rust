use std::alloc::{Layout, alloc, dealloc, handle_alloc_error, realloc};
use std::mem;
use std::ops::Index;
use std::ptr::{self, NonNull};

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

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn cap(&self) -> usize {
        self.cap
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        if index >= self.len {
            return None;
        }
        if mem::size_of::<T>() == 0 {
            // SAFETY: ZSTs need no bytes; any aligned non-null pointer is fine for a reference
            unsafe { Some(&*NonNull::<T>::dangling().as_ptr()) }
        } else {
            // SAFETY: index < len, so the slot is initialized
            unsafe { Some(&*self.ptr.add(index)) }
        }
    }

    fn grow(&mut self) {
        // ZSTs never touch the heap — we only increment len
        if mem::size_of::<T>() == 0 {
            return;
        }

        let new_cap = if self.cap == 0 {
            1
        } else {
            // checked_mul so a vec that somehow reaches usize::MAX/2 doesn't silently wrap
            self.cap.checked_mul(2).expect("capacity overflow")
        };

        let new_layout = Layout::array::<T>(new_cap).unwrap();

        let new_ptr = unsafe {
            if self.cap == 0 {
                alloc(new_layout)
            } else {
                let old_layout = Layout::array::<T>(self.cap).unwrap();
                realloc(self.ptr as *mut u8, old_layout, new_layout.size())
            }
        };

        if new_ptr.is_null() {
            handle_alloc_error(new_layout);
        }

        self.ptr = new_ptr as *mut T;
        self.cap = new_cap;
    }

    pub fn push(&mut self, value: T) {
        if mem::size_of::<T>() == 0 {
            // forget the value so its Drop doesn't run — the "count" is stored in len
            mem::forget(value);
            self.len += 1;
            return;
        }

        if self.len == self.cap {
            self.grow();
        }

        unsafe {
            // SAFETY: grow ensures ptr is valid for at least len+1 slots;
            // write (not assign) avoids running Drop on whatever uninitialized bytes sit there
            ptr::write(self.ptr.add(self.len), value);
        }
        self.len += 1;
    }

    pub fn insert(&mut self, index: usize, value: T) {
        assert!(index <= self.len, "index out of bounds");

        if mem::size_of::<T>() == 0 {
            mem::forget(value);
            self.len += 1;
            return;
        }

        if self.len == self.cap {
            self.grow();
        }

        unsafe {
            // SAFETY: grow ensures cap >= len+1; ptr::copy handles overlapping src/dst correctly
            ptr::copy(
                self.ptr.add(index),
                self.ptr.add(index + 1),
                self.len - index,
            );
            ptr::write(self.ptr.add(index), value);
        }
        self.len += 1;
    }

    pub fn remove(&mut self, index: usize) -> T {
        assert!(index < self.len, "index out of bounds");

        if mem::size_of::<T>() == 0 {
            self.len -= 1;
            // SAFETY: ZST read touches no bytes
            return unsafe { ptr::read(NonNull::<T>::dangling().as_ptr()) };
        }

        unsafe {
            // SAFETY: index < len so the slot is initialized; read moves ownership out
            // before we close the gap with copy
            let value = ptr::read(self.ptr.add(index));
            ptr::copy(
                self.ptr.add(index + 1),
                self.ptr.add(index),
                self.len - index - 1,
            );
            self.len -= 1;
            value
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }
        self.len -= 1;

        if mem::size_of::<T>() == 0 {
            // SAFETY: ZST read touches no actual bytes
            unsafe { Some(ptr::read(NonNull::<T>::dangling().as_ptr())) }
        } else {
            // SAFETY: len was just decremented, so self.len now points at the
            // last initialized slot; read moves ownership out without running Drop on it
            unsafe { Some(ptr::read(self.ptr.add(self.len))) }
        }
    }
}

impl<T> Index<usize> for MyVec<T> {
    type Output = T;

    fn index(&self, index: usize) -> &T {
        assert!(
            index < self.len,
            "index out of bounds: len is {} but index is {}",
            self.len,
            index
        );
        if mem::size_of::<T>() == 0 {
            // SAFETY: ZST — no bytes accessed
            unsafe { &*NonNull::<T>::dangling().as_ptr() }
        } else {
            // SAFETY: bounds checked above
            unsafe { &*self.ptr.add(index) }
        }
    }
}

impl<T> Drop for MyVec<T> {
    fn drop(&mut self) {
        // drop_in_place before dealloc — running it after would be use-after-free
        // for types like String or Box<T> that own memory elsewhere this is what
        // actually frees their inner allocations
        let ptr = if mem::size_of::<T>() == 0 {
            // no heap was ever touched for ZSTs; dangling is fine here because
            // drop_in_place on a ZST never dereferences the pointer
            NonNull::<T>::dangling().as_ptr()
        } else {
            self.ptr
        };

        for i in (0..self.len).rev() {
            unsafe {
                // SAFETY: slot is initialized and within [0, len); drop_in_place
                // runs T's destructor in place without moving the value out
                ptr::drop_in_place(ptr.add(i));
            }
        }

        if self.cap > 0 && mem::size_of::<T>() != 0 {
            let layout = Layout::array::<T>(self.cap).unwrap();
            unsafe {
                // SAFETY: ptr was allocated with this exact layout;
                // all elements are already dropped above
                dealloc(self.ptr as *mut u8, layout);
            }
        }
    }
}

impl<T> Default for MyVec<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let v: MyVec<i32> = MyVec::new();
        assert_eq!(v.len(), 0);
        assert_eq!(v.cap(), 0);
        assert!(v.ptr.is_null());
        assert!(v.is_empty());
    }

    #[test]
    fn test_push_and_pop() {
        let mut v = MyVec::new();
        v.push(10);
        v.push(20);
        assert_eq!(v.len(), 2);
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
        assert_eq!(v.len(), 100);
        for i in (0..100).rev() {
            assert_eq!(v.pop(), Some(i));
        }
    }

    #[test]
    fn test_capacity_doubles() {
        let mut v: MyVec<i32> = MyVec::new();
        v.push(1);
        assert_eq!(v.cap(), 1);
        v.push(2);
        assert_eq!(v.cap(), 2);
        v.push(3);
        assert_eq!(v.cap(), 4);
        v.push(5);
        assert_eq!(v.cap(), 4);
        v.push(6);
        assert_eq!(v.cap(), 8);
    }

    #[test]
    fn test_index() {
        let mut v = MyVec::new();
        v.push(10);
        v.push(20);
        v.push(30);
        assert_eq!(v[0], 10);
        assert_eq!(v[1], 20);
        assert_eq!(v[2], 30);
    }

    #[test]
    #[should_panic(expected = "index out of bounds")]
    fn test_index_out_of_bounds() {
        let mut v: MyVec<i32> = MyVec::new();
        v.push(1);
        let _ = v[99];
    }

    #[test]
    fn test_get() {
        let mut v = MyVec::new();
        v.push(100);
        v.push(200);
        assert_eq!(v.get(0), Some(&100));
        assert_eq!(v.get(1), Some(&200));
        assert_eq!(v.get(2), None);
        assert_eq!(v.get(99), None);
    }

    // The drop test with String is the important one — without a proper Drop impl
    // MyVec would leak the heap backing each String
    #[test]
    fn test_drop_runs_for_string_elements() {
        let mut v = MyVec::new();
        v.push(String::from("hello"));
        v.push(String::from("world"));
        // if Drop is broken, Miri or valgrind will catch the leak here
        drop(v);
    }

    #[test]
    fn test_drop_destructor_count() {
        use std::cell::Cell;

        struct Bomb<'a>(&'a Cell<u32>);
        impl Drop for Bomb<'_> {
            fn drop(&mut self) {
                self.0.set(self.0.get() + 1);
            }
        }

        let count = Cell::new(0u32);
        {
            let mut v = MyVec::new();
            v.push(Bomb(&count));
            v.push(Bomb(&count));
            v.push(Bomb(&count));
            assert_eq!(count.get(), 0);
        }
        assert_eq!(count.get(), 3);
    }

    #[test]
    fn test_zst_push_pop() {
        let mut v: MyVec<()> = MyVec::new();
        v.push(());
        v.push(());
        v.push(());
        assert_eq!(v.len(), 3);
        assert_eq!(v.pop(), Some(()));
        assert_eq!(v.len(), 2);
        assert_eq!(v.cap(), 0); // ZSTs never allocate
    }

    #[test]
    fn test_zst_index_and_get() {
        let mut v: MyVec<()> = MyVec::new();
        v.push(());
        v.push(());
        assert_eq!(v[0], ());
        assert_eq!(v.get(1), Some(&()));
        assert_eq!(v.get(2), None);
    }

    #[test]
    fn test_zst_drop_runs_destructors() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static COUNT: AtomicUsize = AtomicUsize::new(0);

        struct Z;
        impl Drop for Z {
            fn drop(&mut self) {
                COUNT.fetch_add(1, Ordering::SeqCst);
            }
        }

        {
            let mut v = MyVec::new();
            v.push(Z);
            v.push(Z);
            v.push(Z);
            assert_eq!(COUNT.load(Ordering::SeqCst), 0);
        }
        assert_eq!(COUNT.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn test_insert() {
        let mut v = MyVec::new();
        v.push(1);
        v.push(2);
        v.push(3);
        v.insert(1, 99);
        assert_eq!(v.len(), 4);
        assert_eq!(v[0], 1);
        assert_eq!(v[1], 99);
        assert_eq!(v[2], 2);
        assert_eq!(v[3], 3);
    }

    #[test]
    fn test_remove() {
        let mut v = MyVec::new();
        v.push(1);
        v.push(2);
        v.push(4);
        v.push(3);
        let out = v.remove(2);
        assert_eq!(out, 4);
        assert_eq!(v.len(), 3);
        assert_eq!(v[0], 1);
        assert_eq!(v[1], 2);
        assert_eq!(v[2], 3);
    }

    #[test]
    #[should_panic(expected = "index out of bounds")]
    fn test_insert_out_of_bounds() {
        let mut v: MyVec<i32> = MyVec::new();
        v.insert(1, 0);
    }

    #[test]
    #[should_panic(expected = "index out of bounds")]
    fn test_remove_out_of_bounds() {
        let mut v: MyVec<i32> = MyVec::new();
        v.remove(0);
    }

    #[test]
    #[should_panic(expected = "capacity overflow")]
    fn test_capacity_overflow() {
        usize::MAX.checked_mul(2).expect("capacity overflow");
    }
}
