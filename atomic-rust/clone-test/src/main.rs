use std::sync::Arc;
use std::thread;
use std::fmt::Display;

fn dgb<T>(x: &T)
where T: Display {
    println!("x is {}", x);
}

fn f(a: &i32, b: &mut i32) {
    *b = *a + *b;
}

fn main() {
    let a = Arc::new(3);
    thread::scope(|s| {
        s.spawn(|| {
            dgb(&a);
        });
    });
    let b = a.clone();
    thread::spawn(move ||{
        dgb(&b);
    }).join().unwrap();

    dgb(&a);
    dgb(&a);
    let c = String::from("Hello");
    println!("{}", &c);

    let d = 10;
    let mut e = d;
    f(&d, &mut e);
    println!("d is {}, e is {}", d, e);

}
