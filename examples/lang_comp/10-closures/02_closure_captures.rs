fn main() {
    {
        let x = 10;
        {
            let add_x = move |y| { x + y };
            {
                println!("{:?}", add_x(5));
                {
                    let a = 2;
                    {
                        let b = 3;
                        {
                            let calculate = move |n| { n * a + b };
                            {
                                println!("{:?}", calculate(5));
                                fn make_adder(n: i32) -> impl Fn(i32) -> i32 {
                                    move |x| { x + n }
                                }
                                {
                                    let add_five = make_adder(5);
                                    println!("{:?}", add_five(10))
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
