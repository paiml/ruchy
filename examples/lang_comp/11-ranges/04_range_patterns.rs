fn main() {
    for i in 0..5 {
        {
            {
                let value = i * 2;
                println!("{:?}", value)
            }
        }
    }
    {
        let items = ["a", "b", "c", "d"];
        {
            for i in 0..4 {
                { println!("{:?}", items[i as usize].clone()) }
            }
            let mut squares = [0, 0, 0, 0, 0];
            for i in 0..5 {
                { squares[i as usize] = i * i }
            }
            for n in 0..5 {
                { println!("{:?}", squares[n as usize].clone()) }
            }
            let mut countdown = [5, 4, 3, 2, 1];
            for i in 0..5 {
                { println!("{:?}", countdown[i as usize].clone()) }
            }
        }
    }
}
