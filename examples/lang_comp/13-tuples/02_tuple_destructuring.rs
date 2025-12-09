fn main() {
    let (x, y) = (3, 4);
    println!("{:?}", x);
    println!("{:?}", y);
    let (num, text, flag) = (42, "world", false);
    println!("{:?}", num);
    println!("{:?}", text);
    println!("{:?}", flag);
    let ((a, b), c) = ((1, 2), 3);
    println!("{:?}", a);
    println!("{:?}", b);
    println!("{:?}", c);
    {
        let point = (10, 20);
        {
            match point {
                (0, 0) => println!("Origin"),
                (0, y) => println!("On Y-axis at {}", y),
                (x, 0) => println!("On X-axis at {}", x),
                (x, y) => println!("Point at ({}, {})", x, y),
            };
            {
                let triple = (1, 2, 3);
                {
                    let (first, _, third) = triple;
                    println!("{:?}", first);
                    println!("{:?}", third);
                    let mut a = 5;
                    let mut b = 10;
                    {
                        let temp = a;
                        {
                            a = b;
                            b = temp;
                            println!("{:?}", a);
                            println!("{:?}", b)
                        }
                    }
                }
            }
        }
    }
}
