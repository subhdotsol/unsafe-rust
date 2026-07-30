use unsafe_rust::myvec::MyVec;

fn main() {
    let mut v: MyVec<i32> = MyVec::new();
    v.push(10);
    v.push(20);
    v.push(30);
    v.push(40);
    v.push(50);
    println!("len={} cap={}", v.len(), v.cap());
    println!("v[0]={} v[1]={} v[2]={}", v[0], v[1], v[2]);
    println!("get(2)={:?}  get(99)={:?}", v.get(2), v.get(99));

    v.insert(1, 99);
    println!("after insert(1, 99): len={} v[1]={}", v.len(), v[1]);

    let removed = v.remove(1);
    println!("remove(1)={}  len={}", removed, v.len());

    while let Some(x) = v.pop() {
        print!("{x} ");
    }
    println!();
    println!("pop on empty={:?}", v.pop());

    // String elements — verifies Drop calls each String's destructor before freeing the block
    {
        let mut s: MyVec<String> = MyVec::new();
        s.push(String::from("heap-allocated"));
        s.push(String::from("strings"));
        s.push(String::from("are freed on drop"));
        println!("len={}", s.len());
    }

    // ZSTs never allocate; cap stays 0 no matter how many you push
    let mut z: MyVec<()> = MyVec::new();
    z.push(());
    z.push(());
    z.push(());
    println!("zst  len={}  cap={}  pop={:?}", z.len(), z.cap(), z.pop());
}
