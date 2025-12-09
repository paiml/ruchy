fn main() {
    let text = "hello world";
    let upper = text.to_uppercase();
    let lower = text.to_lowercase();
    let len = text.len();
    let trimmed = "  spaces  ".trim();
    let replaced = text.replace("world", "Ruchy");
    println!("hello world");
    println!("{:?}", upper);
    println!("{:?}", lower);
    println!("{:?}", len);
    println!("{:?}", trimmed);
    println!("{:?}", replaced);
}
