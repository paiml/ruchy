fn main() {
    let (x, y) = (10, 20);
    println!("{:?}", x);
    println!("{:?}", y);
    let (a, b, c) = (100, 200, 300);
    println!("{:?}", a);
    println!("{:?}", b);
    println!("{:?}", c);
    let ((p, q), (r, s)) = ((1, 2), (3, 4));
    println!("{:?}", p);
    println!("{:?}", q);
    println!("{:?}", r);
    println!("{:?}", s);
}
