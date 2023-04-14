use std::collections::HashMap;

pub fn parse_header(result: &str) -> (Vec<String>, HashMap<String, String>) {
    let mut lines = result.lines();
    let mut map = HashMap::new();
    let method = lines.next().unwrap();
    let method = method.split(' ').map(|v| v.to_string()).collect();
    for line in lines {
        if line.is_empty() {
            break;
        }
        let mut split = line.split(": ");
        let key = split.next().unwrap().to_lowercase();
        let value = split.next().unwrap().to_string();
        map.insert(key, value);
    }
    (method, map)
}
