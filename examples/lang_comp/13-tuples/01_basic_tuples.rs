fn main() {
    {
        let pair = (1, 2);
        {
            println!("{:?}", pair);
            {
                let mixed = (42, "hello", true);
                {
                    println!("{:?}", mixed);
                    {
                        let first = pair.0;
                        {
                            let second = pair.1;
                            {
                                println!("{:?}", first);
                                println!("{:?}", second);
                                {
                                    let num = mixed.0;
                                    {
                                        let text = mixed.1;
                                        {
                                            let flag = mixed.2;
                                            {
                                                println!("{:?}", num);
                                                println!("{:?}", text);
                                                println!("{:?}", flag);
                                                {
                                                    let unit = ();
                                                    {
                                                        println!("{:?}", unit);
                                                        {
                                                            let single = (100);
                                                            {
                                                                println!("{:?}", single);
                                                                let mut coords = (0, 0);
                                                                coords = (5, 10);
                                                                println!("{:?}", coords)
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
