use hztp::setting;

fn main() -> std::io::Result<()> {
    let ip_list = setting::get_ip().unwrap();
    println!("{ip_list:#?}");

    println!(
        "default_interface_name = {:?}",
        default_net::interface::get_default_interface_name()
    );
    println!(
        "default_interface_name = {:?}",
        default_net::interface::get_local_ipaddr()
    );

    Ok(())
}
