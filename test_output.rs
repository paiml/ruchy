fn main() {
    let result = {
        let greeting = "Hello, Ruchy!";
        greeting
    };
    if let Some(s) = (&result as &dyn std::any::Any).downcast_ref::<String>() {
        println!("{}", s);
    } else if let Some(s) = (&result as &dyn std::any::Any).downcast_ref::<&str>() {
        println!("{}", s);
    } else {
        println!("{:?}", result);
    }
}
