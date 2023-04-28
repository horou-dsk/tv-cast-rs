use std::collections::HashMap;

pub fn parse_header(result: &str) -> Option<(Vec<&str>, HashMap<String, &str>)> {
    let mut lines = result.lines();
    let mut map = HashMap::new();
    let Some(method) = lines.next() else {return None};
    let method = method.split(' ').collect();
    for line in lines {
        if line.is_empty() {
            break;
        }
        if let Some((key, value)) = line.split_once(':') {
            let value = if &value[..1] == " " {
                &value[1..]
            } else {
                value
            };
            map.insert(key.to_lowercase(), value);
        }
    }
    Some((method, map))
}
