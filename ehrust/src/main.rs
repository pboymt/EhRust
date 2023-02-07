use std::error::Error;

use libeh::{add, web};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("Hello, world!");
    let result = add(1, 2);
    println!("result: {}", result);
    let result2 = web().await?;
    println!("{:?}", result2);
    Ok(())
}
