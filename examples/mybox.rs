use unsafe_rust::mybox::MyBox;

fn main() {
    let x = MyBox::new(42);
    println!("value={}", *x);

    let mut n = MyBox::new(10);
    *n += 5;
    println!("after mutation={}", *n);

    // deref coercion kicks in here — MyBox<String> → &String → &str
    let s = MyBox::new(String::from("naina"));
    println!("len={}", s.len());

    {
        let b = MyBox::new(String::from("dropped at end of scope"));
        println!("{}", *b);
    }

    // Vec inside Box — when this drops, Vec's destructor runs first,
    // then MyBox frees the slot that held it
    let big = MyBox::new(vec![1, 2, 3, 4, 5]);
    println!("{:?}", *big);
}
