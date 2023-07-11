pub mod exo_golomb;
mod h264_dcoder;
pub mod native_obj;

use std::{cell::UnsafeCell, str::FromStr, sync::Arc};

use airplay2_protocol::{
    airplay::{
        airplay_consumer::AirPlayConsumer, lib::audio_stream_info::CompressionType, AirPlayConfig,
    },
    airplay_bonjour::AirPlayBonjour,
    control_handle::ControlHandle,
    net::server::Server,
};
use gst::{prelude::*, Caps};
use gst_app::{AppSrc, AppStreamType};
use jni::{
    objects::{GlobalRef, JClass, JObject},
    JNIEnv,
};
use ndk::native_window::NativeWindow;

static mut VIDEO_CONSUMER: Option<Arc<VideoConsumer>> = None;
static mut NATIVE_WINDOW: Option<NativeWindow> = None;
static mut G_OBJ: Option<GlobalRef> = None;

use crate::{G_JVM, G_SERVICE};

use self::h264_dcoder::H264Decoder;

pub struct VideoConsumer {
    alac: (gst::Pipeline, AppSrc, gst::Element),
    aac_eld: (gst::Pipeline, AppSrc, gst::Element),
    audio_compression_type: UnsafeCell<CompressionType>,
    h264_decoder: H264Decoder,
}

unsafe impl Sync for VideoConsumer {}

impl Default for VideoConsumer {
    fn default() -> Self {
        Self::new()
    }
}

impl VideoConsumer {
    fn new() -> Self {
        let caps = Caps::from_str("audio/x-alac,mpegversion=(int)4,channels=(int)2,rate=(int)48000,stream-format=raw,codec_data=(buffer)00000024616c616300000000000001600010280a0e0200ff00000000000000000000ac44").unwrap();
        let alac_pipeline = gst::Pipeline::default();

        let alac_appsrc = AppSrc::builder()
            .is_live(true)
            .stream_type(AppStreamType::Stream)
            .caps(&caps)
            .format(gst::Format::Time)
            .build();

        let alac_volume = gst::ElementFactory::make("volume").build().unwrap();
        let avdec_alac = gst::ElementFactory::make("avdec_alac").build().unwrap();
        let audioconvert = gst::ElementFactory::make("audioconvert").build().unwrap();
        let audioresample = gst::ElementFactory::make("audioresample").build().unwrap();
        let autoaudiosink = gst::ElementFactory::make("autoaudiosink")
            .property("sync", false)
            .build()
            .unwrap();

        alac_pipeline
            .add_many(&[
                alac_appsrc.upcast_ref(),
                &alac_volume,
                &avdec_alac,
                &audioconvert,
                &audioresample,
                &autoaudiosink,
            ])
            .unwrap();
        gst::Element::link_many(&[
            alac_appsrc.upcast_ref(),
            &avdec_alac,
            &audioconvert,
            &alac_volume,
            &audioresample,
            &autoaudiosink,
        ])
        .unwrap();

        let caps = Caps::from_str("audio/mpeg,mpegversion=(int)4,channnels=(int)2,rate=(int)44100,stream-format=raw,codec_data=(buffer)f8e85000").unwrap();
        let aac_eld_pipeline = gst::Pipeline::default();

        let aac_eld_appsrc = AppSrc::builder()
            .is_live(true)
            .stream_type(AppStreamType::Stream)
            .caps(&caps)
            .format(gst::Format::Time)
            .build();
        let aac_eld_volume = gst::ElementFactory::make("volume").build().unwrap();
        let avdec_aac = gst::ElementFactory::make("avdec_aac").build().unwrap();
        let audioconvert = gst::ElementFactory::make("audioconvert").build().unwrap();
        let audioresample = gst::ElementFactory::make("audioresample").build().unwrap();
        let autoaudiosink = gst::ElementFactory::make("autoaudiosink")
            .property("sync", false)
            .build()
            .unwrap();
        aac_eld_pipeline
            .add_many(&[
                aac_eld_appsrc.upcast_ref(),
                &avdec_aac,
                &audioconvert,
                &aac_eld_volume,
                &audioresample,
                &autoaudiosink,
            ])
            .unwrap();
        gst::Element::link_many(&[
            aac_eld_appsrc.upcast_ref(),
            &avdec_aac,
            &audioconvert,
            &aac_eld_volume,
            &audioresample,
            &autoaudiosink,
        ])
        .unwrap();

        Self {
            alac: (alac_pipeline, alac_appsrc, alac_volume),
            aac_eld: (aac_eld_pipeline, aac_eld_appsrc, aac_eld_volume),
            audio_compression_type: CompressionType::Alac.into(),
            h264_decoder: H264Decoder::default(),
        }
    }
}

impl AirPlayConsumer for VideoConsumer {
    fn on_video(&self, bytes: Vec<u8>) {
        if let Err(err) = self.h264_decoder.decode_buf(&bytes) {
            log::error!("Error pushing buffer: {:?}", err);
        }
    }

    fn on_video_format(
        &self,
        video_stream_info: airplay2_protocol::airplay::lib::video_stream_info::VideoStreamInfo,
    ) {
        log::info!(
            "OnVideo Format... {:?}",
            video_stream_info.get_stream_connection_id()
        );
        let jvm = unsafe { G_JVM.as_ref().unwrap() };
        let mut env = jvm.attach_current_thread().unwrap();
        env.call_method(
            unsafe { G_SERVICE.as_ref().unwrap() },
            "startAirplayActivity",
            "()V",
            &[],
        )
        .unwrap();
        if let Some(surface) = unsafe { NATIVE_WINDOW.as_ref() } {
            if let Err(err) = self.h264_decoder.set_surface(surface) {
                log::error!("Error setting surface: {:?}", err);
            }
        }
        if let Err(err) = self.h264_decoder.start_decode() {
            log::error!("Error playing: {:?}", err);
        }
    }

    fn on_video_src_disconnect(&self) {
        log::info!("OnVideo Disconnect...");
        if let Err(err) = self.h264_decoder.stop_decode() {
            log::error!("Error stopping: {:?}", err);
        }
    }

    fn on_audio_format(
        &self,
        audio_stream_info: airplay2_protocol::airplay::lib::audio_stream_info::AudioStreamInfo,
    ) {
        log::info!(
            "OnAudio Format......type = {:?}",
            audio_stream_info.compression_type
        );
        unsafe { *self.audio_compression_type.get() = audio_stream_info.compression_type };
        self.alac
            .0
            .set_state(gst::State::Playing)
            .expect("Unable to set the pipeline to the `Playing` state");
        self.aac_eld
            .0
            .set_state(gst::State::Playing)
            .expect("Unable to set the pipeline to the `Playing` state");
    }

    fn on_audio(&self, bytes: Vec<u8>) {
        let buffer = gst::Buffer::from_mut_slice(bytes);
        match unsafe { &*self.audio_compression_type.get() } {
            CompressionType::Alac => {
                self.alac.1.push_buffer(buffer).ok();
            }
            _ => {
                self.aac_eld.1.push_buffer(buffer).ok();
            }
        }
    }

    fn on_audio_src_disconnect(&self) {
        log::info!("OnAudio Disconnect...");
        self.alac
            .0
            .set_state(gst::State::Null)
            .expect("Unable to set the pipeline to the `Null` state");
        self.aac_eld
            .0
            .set_state(gst::State::Null)
            .expect("Unable to set the pipeline to the `Null` state");
    }

    fn on_volume(&self, volume: f32) {
        let volume = volume / 30.0 + 1.0;
        match unsafe { &*self.audio_compression_type.get() } {
            CompressionType::Alac => {
                self.alac.2.set_property("volume", volume as f64);
            }
            _ => {
                self.aac_eld.2.set_property("volume", volume as f64);
            }
        }
    }
}

/// # Safety
pub unsafe fn native_surface_init(env: JNIEnv, _class: JClass, obj: JObject, surface: JObject) {
    let surface = ndk::native_window::NativeWindow::from_surface(env.get_raw(), surface.into_raw());
    G_OBJ = Some(env.new_global_ref(obj).unwrap());
    NATIVE_WINDOW = Some(surface.unwrap());
}

/// # Safety
pub unsafe fn native_surface_finalize(_env: JNIEnv, _class: JClass) {
    NATIVE_WINDOW.take();
    G_OBJ.take();
}

pub async fn airplay_run(name: String) -> std::io::Result<()> {
    let airplay_config = AirPlayConfig {
        server_name: name.clone(),
        width: 1920,
        height: 1080,
        fps: 60,
    };
    let video_consumer = Arc::new(VideoConsumer::default());
    unsafe {
        VIDEO_CONSUMER = Some(video_consumer.clone());
    }
    // tx.send(()).unwrap();
    let server = Server::bind_default(ControlHandle::new(
        airplay_config,
        video_consumer.clone(),
        video_consumer,
    ))
    .await;

    // pin码认证功能缺失...
    let _air = AirPlayBonjour::new(&name, server.port, false);

    log::info!("Airplay Server Starting... port = {}", server.port);
    server.run().await?;
    Ok(())
}
