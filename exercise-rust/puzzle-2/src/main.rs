use std::io::stdin;
fn main() {
    println!("What is 3+2? Type your answer and press Enter.");
    let mut input = String::new();
    stdin()
        .read_line(&mut input)
        .expect("Unable to read stndard input");

    if input.trim() == "5" {
        println!("Correct!");
    } else {
        println!("Incorrect!");
    }
}
