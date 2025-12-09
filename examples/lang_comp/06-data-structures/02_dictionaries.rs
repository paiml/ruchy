fn main() {
    let person = {
        let mut map: std::collections::BTreeMap<String, String> = std::collections::BTreeMap::new();
        map.insert("name".to_string(), ("Alice").to_string());
        map.insert("age".to_string(), ("30").to_string());
        map.insert("city".to_string(), ("NYC").to_string());
        map
    };
    let settings = {
        let mut map: std::collections::BTreeMap<String, String> = std::collections::BTreeMap::new();
        map.insert("theme".to_string(), ("dark").to_string());
        map.insert("language".to_string(), ("en").to_string());
        map
    };
    println!("{:?}", person);
    println!("{:?}", settings);
}
