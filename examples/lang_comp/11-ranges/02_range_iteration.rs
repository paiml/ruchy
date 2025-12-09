fn main() {
    let mut sum = 0;
    for i in 1..6 {
        { sum = sum + i }
    }
    println!("{:?}", sum);
    for i in 0..3 {
        {
            for j in 0..2 {
                { println!("{:?}", i * 10 + j) }
            }
        }
    }
    fn print_range(start: i32, end: i32) {
        for n in start..end {
            { println!("{:?}", n) }
        }
    }
    print_range(5, 8)
}
