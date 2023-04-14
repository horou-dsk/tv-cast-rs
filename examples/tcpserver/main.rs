fn main() {
    let msg = [0; 1024];
    println!("{}", String::from_utf8_lossy(&msg));

    let buf = "好玩好玩水电费卡圣诞福利温热12300021401阿萨德        ".as_bytes();
    println!("{:?}", buf);
}
