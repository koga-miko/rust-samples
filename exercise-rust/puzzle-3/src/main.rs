fn main() {
    let x : u64 = 4_294_967_296;
    let y = x as u32;
    if x == y as u64 {
        println!("x eaquals y.");
    } else {
        println!("x does not equal y.");
    }

    let x = u32::max_value();
    println!("x is {}", x);
    let z : u64 = x.into();
    println!("z is {}", z);

    let x : u32 = 4_294_967_296u64.try_into().unwrap();
    println!("x is {}", x);
}
