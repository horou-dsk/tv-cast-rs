use std::collections::HashMap;

pub fn parse_header(result: &str) -> Option<(Vec<String>, HashMap<String, String>)> {
    let mut lines = result.lines();
    let mut map = HashMap::new();
    let Some(method) = lines.next() else {return None};
    let method = method.split(' ').map(|v| v.to_string()).collect();
    for line in lines {
        if line.is_empty() {
            break;
        }
        let mut split = line.split(": ");
        let Some(key) = split.next().map(|v| v.to_lowercase()) else {return None};
        let Some(value) = split.next().map(|v| v.to_string()) else {return None};
        map.insert(key, value);
    }
    Some((method, map))
}
