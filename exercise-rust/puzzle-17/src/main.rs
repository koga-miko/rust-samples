my_vec.push("Hello");
fn main() {
    let mut my_vec = Vec::with_capacity(1);
    println!("{}", my_vec.capacity());
    my_vec.push("World");
    println!("{}", my_vec.capacity());
}
