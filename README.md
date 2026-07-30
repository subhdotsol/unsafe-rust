# unsafe-rust

Reimplementing core Rust data structures from scratch using raw pointers, manual allocations, and `unsafe` blocks — no standard library collections allowed.

The goal is to understand what the standard library actually does under the hood: how heap memory is requested, how ownership is tracked without the borrow checker's help, how atomics replace locks for concurrent access, and what `Drop`, `Deref`, and `Send`/`Sync` really mean at the pointer level.

---

## Implemented

### `MyBox<T>` — owned heap pointer

A minimal reimplementation of `Box<T>`.

```
MyBox::new(value)     allocate one T on the heap, write the value
*box                  Deref → shared reference to the inner T
*box = new_val        DerefMut → mutable reference to the inner T
drop(box)             drop_in_place runs T's destructor, then dealloc frees the memory
```

**Unsafe patterns used:**
- `alloc` / `dealloc` with a manually computed `Layout`
- `NonNull<T>` instead of a raw `*mut T` to enforce non-null invariant at the type level
- `ptr::drop_in_place` to run `T`'s destructor before freeing the allocation
- `Deref` / `DerefMut` trait impls to enable `*` syntax

---

### `MyVec<T>` — growable heap array

A growable, heap-allocated array with amortized O(1) push.

```
MyVec::new()          start empty, no allocation
push(value)           write to the next slot; grow (double capacity) if full
pop()                 move the last element out; return None if empty
get(index)            bounds-checked read, returns Option<&T>
v[index]              Index trait — panics on out-of-bounds
insert(index, value)  shift right and write into the gap
remove(index)         read out, shift left, decrement len
```

**Unsafe patterns used:**
- `alloc` for initial allocation, `realloc` for growth
- `ptr::write` to initialize uninitialized slots without reading what was there
- `ptr::read` to move ownership out of a slot without running Drop on it
- Pointer arithmetic via `ptr.add(offset)` for direct slot addressing

---

## Planned

| Structure | Description | Key technique |
|---|---|---|
| `AppendVec<T>` | Fixed-capacity, single-writer multi-reader vec | `AtomicUsize` len with Release/Acquire pairs; no realloc to keep reader pointers valid |
| `MyRc<T>` | Single-threaded reference-counted pointer | Non-atomic reference count; `Deref` + `Drop` |
| `MyArc<T>` | Thread-safe reference-counted pointer | `AtomicUsize` reference count; `Send` + `Sync` |
| `MyCell<T>` / `MyRefCell<T>` | Interior mutability primitives | `UnsafeCell`, runtime borrow tracking |
| `MyLinkedList<T>` | Doubly-linked list | Raw `*mut Node<T>` pointers; manual link/unlink |

---

## Running

```bash
cargo test              # run all tests
cargo test mybox        # test only MyBox
cargo test myvec        # test only MyVec
```

No external dependencies. `edition = "2024"`.
