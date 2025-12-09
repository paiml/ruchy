fn main() {
    {
        let square = move |x| { x * x };
        {
            println!("{:?}", square(5));
            {
                let process = move |n| {
                    {
                        let doubled = n * 2;
                        {
                            let added = doubled + 10;
                            added
                        }
                    }
                };
                {
                    println!("{:?}", process(15));
                    fn make_multiplier(factor: i32) -> impl Fn(i32) -> i32 {
                        move |x| { x * factor }
                    }
                    {
                        let times_three = make_multiplier(3);
                        println!("{:?}", times_three(7))
                    }
                }
            }
        }
    }
}
