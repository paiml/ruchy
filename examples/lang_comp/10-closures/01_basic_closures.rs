fn main() {
    {
        let greet = move || { println!("Hello!") };
        {
            greet();
            {
                let double = move |x| { x * 2 };
                {
                    println!("{:?}", double(21));
                    {
                        let add = move |a, b| { a + b };
                        {
                            println!("{:?}", add(10, 20));
                            {
                                let multiply = move |x, y| {
                                    {
                                        let result = x * y;
                                        result
                                    }
                                };
                                println!("{:?}", multiply(3, 4))
                            }
                        }
                    }
                }
            }
        }
    }
}
