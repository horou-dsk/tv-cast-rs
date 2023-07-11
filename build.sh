if [ -n "$1" ] && [ "$1" = "--release" ]
then
  cargo build --release --target=armv7-linux-androideabi
  cp target/armv7-linux-androideabi/release/libhztp.so /mnt/d/Code/Work/Projects/YC/TvCast/app/src/main/jniLibs/armeabi-v7a/libhztp.so
else
  cargo build --target=armv7-linux-androideabi
  llvm-strip target/armv7-linux-androideabi/debug/libhztp.so
  cp target/armv7-linux-androideabi/debug/libhztp.so /mnt/d/Code/Work/Projects/YC/TvCast/app/src/main/jniLibs/armeabi-v7a/libhztp.so
fi