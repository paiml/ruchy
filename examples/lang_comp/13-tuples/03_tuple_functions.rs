fn main() {
    fn get_coordinates() -> (i32, i32) {
        (100, 200)
    }
    {
        let coords = get_coordinates();
        {
            println!("{:?}", coords);
            fn print_point(point: (i32, i32)) {
                println!("Point: ({}, {})", point.0, point.1)
            }
            print_point((5, 10));
            fn calculate_distance(p1: (i32, i32), p2: (i32, i32)) -> i32 {
                {
                    let dx = p2.0 - p1.0;
                    {
                        let dy = p2.1 - p1.1;
                        dx * dx + dy * dy
                    }
                }
            }
            {
                let dist = calculate_distance((0, 0), (3, 4));
                {
                    println!("{:?}", dist);
                    fn get_quotient_and_remainder(
                        dividend: i32,
                        divisor: i32,
                    ) -> (i32, i32) {
                        (dividend / divisor, dividend % divisor)
                    }
                    {
                        let result = get_quotient_and_remainder(17, 5);
                        {
                            let q = result.0;
                            {
                                let r = result.1;
                                {
                                    println!("17 / 5 = {} remainder {}", q, r);
                                    fn create_user_tuple() -> (i32, String, bool) {
                                        let name: String = "Alice".to_string();
                                        {
                                            let tuple = (42, name, true);
                                            tuple
                                        }
                                    }
                                    {
                                        let info = create_user_tuple();
                                        {
                                            let id = info.0;
                                            {
                                                let username = info.1;
                                                {
                                                    let active = info.2;
                                                    {
                                                        println!("{:?}", id);
                                                        println!("{:?}", username);
                                                        println!("{:?}", active)
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
