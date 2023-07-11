fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 获取当前目标架构
    let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    if target_arch == "aarch64" {
        println!(
            r"cargo:rustc-link-search=native=/home/krysha/projects/tmp/android_gst/obj/local/arm64-v8a"
        );
    } else {
        println!(
            r"cargo:rustc-link-search=native=/home/krysha/projects/tmp/android_gst/obj/local/armeabi-v7a"
        );
    }
    println!("cargo:rustc-link-lib=dylib=gstreamer_android");
    Ok(())
}
