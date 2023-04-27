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
        if let Some((key, value)) = line.split_once(':') {
            let key = key.to_lowercase();
            let value = if &value[..1] == " " {
                value[1..].to_string()
            } else {
                value.to_string()
            };
            map.insert(key, value);
        }
    }
    Some((method, map))
}
