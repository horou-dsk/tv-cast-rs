fn main() {
    let interfaces = pnet_datalink::interfaces();
    for interface in interfaces {
        println!("{:?}", interface);
    }
}
