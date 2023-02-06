use libeh::{add};

#[tokio::main]
async fn main() {
    println!("Hello, world!");
    let result = add(1, 2);
    println!("result: {}", result);
}
