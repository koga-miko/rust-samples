fn main() {
    let mut counter : i8 = 0;
    loop {
        println!("{}", counter);
        if let Some(n) = counter.checked_add(1) {
            counter  = n;
        } else {
            println!("Counter overflowed!");
            break;
        }
    }
}
