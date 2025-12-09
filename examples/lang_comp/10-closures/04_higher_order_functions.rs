fn main() {
    fn apply(f: impl Fn(i32) -> i32, x: i32) -> i32 {
        f(x)
    }
    {
        let double = move |n| { n * 2 };
        {
            println!("{:?}", apply(double, 21));
            fn compose(f: impl Fn(i32) -> i32, g: impl Fn(i32) -> i32, x: i32) -> i32 {
                f(g(x))
            }
            {
                let add_one = move |n| { n + 1 };
                {
                    let times_two = move |n| { n * 2 };
                    {
                        println!("{:?}", compose(add_one, times_two, 5));
                        fn make_counter(start: i32) -> impl Fn(i32) -> i32 {
                            move |increment| { start + increment }
                        }
                        {
                            let counter = make_counter(10);
                            {
                                println!("{:?}", counter(5));
                                println!("{:?}", counter(3))
                            }
                        }
                    }
                }
            }
        }
    }
}
