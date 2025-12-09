fn main() {
    fn find_user(id: i32) -> Option<String> {
        if id == 1 {
            Some("Alice".to_string())
        } else {
            if id == 2 { Some("Bob".to_string()) } else { None }
        }
    }
    {
        let user1 = find_user(1);
        {
            match user1 {
                Some(name) => println!("{:?}", name),
                None => println!("Not found"),
            };
            {
                let user2 = find_user(999);
                {
                    match user2 {
                        Some(name) => println!("{:?}", name),
                        None => println!("Not found"),
                    };
                    fn find_first_even(numbers: [i32; 5]) -> Option<i32> {
                        for n in 0..5 {
                            {
                                if numbers[n as usize].clone() % 2 == 0 {
                                    return Some(numbers[n as usize].clone());
                                }
                            }
                        }
                        None
                    }
                    {
                        let nums1 = [1, 3, 5, 6, 7];
                        {
                            match find_first_even(nums1) {
                                Some(n) => println!("{:?}", n),
                                None => println!("No even"),
                            };
                            {
                                let nums2 = [1, 3, 5, 7, 9];
                                match find_first_even(nums2) {
                                    Some(n) => println!("{:?}", n),
                                    None => println!("No even"),
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
