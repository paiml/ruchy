fn main() {
    {
        let range1 = 0..5;
        {
            for i in range1 {
                { println!("{:?}", i) }
            }
            {
                let range2 = 10..12;
                {
                    let range3 = 20..22;
                    {
                        for n in range2 {
                            { println!("{:?}", n) }
                        }
                        for m in range3 {
                            { println!("{:?}", m) }
                        }
                        {
                            let start = 3;
                            {
                                let end = 6;
                                {
                                    let dynamic_range = start..end;
                                    for x in dynamic_range {
                                        { println!("{:?}", x) }
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
