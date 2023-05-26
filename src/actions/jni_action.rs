use jni::{
    objects::{GlobalRef, JObject, JValueGen},
    JavaVM,
};
use serde::{Deserialize, Serialize};

use super::{avtransport::SetAVTransportURI, renderingcontrol::SetVolume};

#[derive(Debug, Deserialize, Serialize)]
pub struct EachAction<T = u8> {
    pub action: String,
    pub data: Option<T>,
}

impl EachAction<u8> {
    pub fn only_action(action: &str) -> Self {
        Self {
            action: action.into(),
            data: None,
        }
    }
}

impl<T> EachAction<T> {
    pub fn new(action: &str, data: T) -> Self {
        Self {
            action: action.into(),
            data: Some(data),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransportState {
    pub current_transport_state: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PositionInfo {
    pub track_duration: String,
    pub rel_time: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SeekTarget {
    pub target: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AVTransportURI {
    pub uri: String,
    pub uri_meta: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VolumeInfo {
    pub current_volume: i32,
}

pub struct AVTransportAction {
    jvm: JavaVM,
    obj: GlobalRef,
}

impl AVTransportAction {
    pub fn new(jvm: JavaVM, obj: GlobalRef) -> Self {
        Self { jvm, obj }
    }

    fn jv_to_string(&self, jv: JValueGen<JObject>) -> jni::errors::Result<String> {
        let jb = jv.l()?;
        let mut env = self.jvm.get_env()?;
        let result = env.get_string(&jb.into())?.into();
        Ok(result)
    }

    pub fn set_uri(&mut self, av_uri: SetAVTransportURI) -> jni::errors::Result<()> {
        let mut env = self.jvm.attach_current_thread()?;
        let uri = env.new_string(av_uri.uri).unwrap();
        let meta = env.new_string(av_uri.uri_meta_data).unwrap();
        env.call_method(
            &self.obj,
            "setUri",
            "(Ljava/lang/String;Ljava/lang/String;)V",
            &[JValueGen::Object(&uri), JValueGen::Object(&meta)],
        )?;
        Ok(())
    }

    pub fn get_transport_info(&mut self) -> jni::errors::Result<TransportState> {
        let mut env = self.jvm.attach_current_thread()?;
        let jv = env.call_method(&self.obj, "getTransportInfo", "()Ljava/lang/String;", &[])?;
        let result = self.jv_to_string(jv)?;
        Ok(serde_json::from_str(&result).unwrap())
    }

    pub fn get_position(&mut self) -> jni::errors::Result<PositionInfo> {
        let mut env = self.jvm.attach_current_thread()?;
        let jv = env.call_method(&self.obj, "getPosition", "()Ljava/lang/String;", &[])?;
        let result = self.jv_to_string(jv)?;
        Ok(serde_json::from_str(&result).unwrap())
    }

    pub fn get_volume(&mut self) -> jni::errors::Result<VolumeInfo> {
        let mut env = self.jvm.attach_current_thread()?;
        let jv = env.call_method(&self.obj, "getVolume", "()Ljava/lang/String;", &[])?;
        let result = self.jv_to_string(jv)?;
        Ok(serde_json::from_str(&result).unwrap())
    }

    pub fn play(&mut self) -> jni::errors::Result<()> {
        let mut env = self.jvm.attach_current_thread()?;
        env.call_method(&self.obj, "play", "()V", &[]).ok();
        Ok(())
    }

    pub fn stop(&mut self) -> jni::errors::Result<()> {
        let mut env = self.jvm.attach_current_thread()?;
        env.call_method(&self.obj, "stop", "()V", &[]).ok();
        Ok(())
    }

    pub fn pause(&mut self) -> jni::errors::Result<()> {
        let mut env = self.jvm.attach_current_thread()?;
        env.call_method(&self.obj, "pause", "()V", &[]).ok();
        Ok(())
    }

    pub fn seek(&mut self, target: String) -> jni::errors::Result<()> {
        let mut env = self.jvm.attach_current_thread()?;
        let target = env.new_string(target).unwrap();
        env.call_method(
            &self.obj,
            "seek",
            "(Ljava/lang/String;)V",
            &[(&target).into()],
        )?;
        Ok(())
    }

    pub fn set_mute(&mut self, mute: bool) -> jni::errors::Result<()> {
        let mut env = self.jvm.attach_current_thread()?;
        env.call_method(&self.obj, "seek", "(Z)V", &[mute.into()])?;
        Ok(())
    }

    pub fn set_volume(&mut self, vol: SetVolume) -> jni::errors::Result<()> {
        let mut env = self.jvm.attach_current_thread()?;
        let channel = env.new_string(vol.channel).unwrap();
        env.call_method(
            &self.obj,
            "setVolume",
            "(Ljava/lang/String;I)V",
            &[(&channel).into(), vol.desired_volume.into()],
        )?;
        Ok(())
    }
}
