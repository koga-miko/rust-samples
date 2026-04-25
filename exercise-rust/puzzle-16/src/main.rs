fn double_it<T>(n: T) -> T 
where T: std::ops::Mul<Output = T> + From<i32> {
    n * 2.into()
}
fn main() {
    println!("double_it(2) is {}", double_it(2));
}
