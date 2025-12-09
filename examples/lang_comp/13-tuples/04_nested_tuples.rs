fn main() {
    {
        let nested = ((1, 2), (3, 4));
        {
            println!("{:?}", nested);
            {
                let first_pair = nested.0;
                {
                    let second_pair = nested.1;
                    {
                        println!("{:?}", first_pair);
                        println!("{:?}", second_pair);
                        {
                            let value = nested.0.1;
                            {
                                println!("{:?}", value);
                                {
                                    let deep = (((1, 2), 3), 4);
                                    {
                                        println!("{:?}", deep);
                                        {
                                            let pair1 = (10, 20);
                                            {
                                                let pair2 = (30, 40);
                                                {
                                                    let nested2 = (pair1, pair2);
                                                    {
                                                        let (a, b) = nested2.0;
                                                        let (c, d) = nested2.1;
                                                        println!("{:?}", a);
                                                        println!("{:?}", b);
                                                        println!("{:?}", c);
                                                        println!("{:?}", d);
                                                        {
                                                            let complex = (1, (2, 3), ((4, 5), 6));
                                                            {
                                                                println!("{:?}", complex);
                                                                {
                                                                    let x = complex.0;
                                                                    {
                                                                        let y = complex.1.0;
                                                                        {
                                                                            let z = complex.2.0.1;
                                                                            {
                                                                                println!("{:?}", x);
                                                                                println!("{:?}", y);
                                                                                println!("{:?}", z)
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
                    }
                }
            }
        }
    }
}
