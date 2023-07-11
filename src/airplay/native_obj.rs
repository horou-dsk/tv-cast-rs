/* use jni::{
    objects::{GlobalRef, JValueGen},
    JavaVM,
};

use crate::BUFFER_CHANNEL;

pub struct NativeObj {
    jvm: JavaVM,
    obj: GlobalRef,
}

impl NativeObj {
    pub fn new(jvm: JavaVM, obj: GlobalRef) -> Self {
        Self { jvm, obj }
    }

    // fn get_env(&self) -> jni::errors::Result<jni::JNIEnv> {
    //     match self.jvm.get_env() {
    //         Ok(env) => Ok(env),
    //         Err(_) => {
    //             let env = self.jvm.attach_current_thread()?;
    //             Ok(env.unsafe_clone())
    //         }
    //     }
    // }

    pub fn push_buffer(&self, buf: Vec<u8>) -> Result<(), Box<dyn std::error::Error>> {
        BUFFER_CHANNEL.0.send(buf)?;
        // let mut env = self.jvm.attach_current_thread()?;
        // let buf = env.byte_array_from_slice(&buf)?;
        // env.call_method(&self.obj, "onBuffer", "([B)V", &[JValueGen::Object(&buf)])?;
        Ok(())
    }

    pub fn playing(&self) -> jni::errors::Result<()> {
        let mut env = self.jvm.attach_current_thread()?;
        env.call_method(&self.obj, "playingVideo", "()V", &[])?;
        Ok(())
    }

    pub fn stop(&self) -> jni::errors::Result<()> {
        let mut env = self.jvm.attach_current_thread()?;
        env.call_method(&self.obj, "stopVideo", "()V", &[])?;
        Ok(())
    }
}
 */
