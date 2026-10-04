// what is arc ?
// Arc<T> is a smart pointer that lets multiple owners share one T, and automatically destroys the
// T when the last strong owner goes away. Its reference count is atomic so those owners can safely exist across threads.
//
//
// - a value normally has one owner at a time
// what if i want multiple owners ?
// you can use clone but then you will have two seperate strings
//
// so here comes arc
// its job is to allow multiple owners of a value
//
// we write
// use std::sync::Arc;
// let a = Arc::new(String::from("Kyral"));
// let b = Arc::clone(&a);
// now there is one string but there are two owners
//
// why is it calle reference counted ?
// let a = Arc::new(String::from("Kyral"));  // strong_count = 1 , vaule = "kyral"
// let b = Arc::clone(&a);  // strong_count = 2 , value = "kyral"
//
// what happens if the owners disappears ?
// let a = Arc::new(42);
// let b = Arc::clone(&a);
// let c = Arc::clone(&a);
// strong_count = 3 -> drop (a)
//
// so why no use RC ?
// use std::rc::Rc; // also provide multiple ownerships
// Rc<T> and Arc<T> // solves similar problem but the important difference is threads
// Rc is single threaded , Arc is multi threaded
//
// what exactly does arc::clone do ?
// a and b points to same memory location
// it means give me another owner of this allocation
//
// why is the data on the heap ?
// the arc itself is a small handle and the actual T lives in a shared heap allocation
// that's what allows multiple Arcs to  point to the same value
//
// why do we need to refrence count ?
// to answer when is it safe to destory the value
//
// why "atomic" specifically
// what if two threads execute drop at the same time ? the operation could race so arc uses an atomic counter
// which makes are appropriate for sharing ownership across threads

// arc doesn not make T thread safe
// this is a very common misconception
// it does not automatically mean you can safely mutate T from multiple threads
// for shared mutation you might need things like Mutex and Rwlock
// arc -> shared ownership , mutex -> shared mutation

// Note : What to implement
// myArc::new(value);
// myArc::clone(&arc);
// Deref
// Drop

use std::{ops::Deref, ptr::NonNull};

struct MyArc<T> {
    ptr: NonNull<Inner<T>>,
}

struct Inner<T> {
    count: usize,
    value: T,
}

impl<T> MyArc<T> {
    pub fn new(value: T) -> Self {
        let inner = Box::new(Inner { count: 1, value });

        Self {
            ptr: NonNull::from(Box::leak(inner)), // leaks means dont automatically destroy this box and leave the
                                                  // allocation alive
        }
    }

    pub fn clone(&self) -> Self {
        unsafe {
            (*self.ptr.as_ptr()).count += 1;

            Self { ptr: self.ptr }
        }
    }

    pub fn strong_count(&self) -> usize {
        unsafe { self.ptr.as_ref().count }
    }
}

impl<T> Deref for MyArc<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &(*self.ptr.as_ptr()).value }
    }
}

// impl<T> Drop for MyArc<T> {
//     fn drop(&mut self) {
//         unsafe {
//             (*self.ptr.as_ptr()).count -= 1;
//             if (*self.ptr.as_ptr()).count == 0 {
//                 Box::from_raw(self.ptr.as_ptr());
//             }
//         }
//     }
// }

impl<T> Drop for MyArc<T> {
    fn drop(&mut self) {
        unsafe {
            let inner = self.ptr.as_ptr();
            (*inner).count -= 1;
            if (*inner).count == 0 {
                Box::from_raw(inner);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::clone;

    use super::*;

    #[test]
    fn test_new_arc() {
        let arc = MyArc::new(42);
        assert_eq!(arc.strong_count(), 1);
    }

    #[test]
    fn test_clone_arc() {
        let arc = MyArc::new(42);
        let cloned = arc.clone();
        assert_eq!(arc.strong_count(), 2);
        assert_eq!(cloned.strong_count(), 2);
    }

    #[test]
    fn test_drop_arc() {
        let arc = MyArc::new(42);
        let cloned = arc.clone();
        assert_eq!(arc.strong_count(), 2);
        drop(arc);
        assert_eq!(cloned.strong_count(), 1);
        drop(cloned);
    }

    #[test]
    fn test_deref_arc() {
        let arc = MyArc::new(42);
        assert_eq!(*arc, 42);
    }
}

// what does API of arc contain ?
// 1. creating an arc
// 2. sharing it
// 3. accessing the value
// 4. destroying the owner
// 5. Looking at the count
// 6. getting a mutable access
// 7. taking T back -> Arc::try_unwrap(arc) // if i am a strong owner can i take T out without cloning it
// 8. weak reference -> distinguish  lifetime of T from lifetime of the allocation/control block
// 9. raw pointer APIs -> Arc::into_raw(arc), Arc::from_raw(ptr), Arc::as_ptr(&arc)
