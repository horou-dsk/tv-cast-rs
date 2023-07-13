pub mod exo_golomb;
mod media_decoder;
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
use crossbeam::channel;
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

use self::media_decoder::H264Decoder;

pub struct VideoConsumer {
    alac: (gst::Pipeline, AppSrc, gst::Element),
    aac_eld: (gst::Pipeline, AppSrc, gst::Element),
    audio_compression_type: UnsafeCell<CompressionType>,
    h264_decoder: UnsafeCell<Option<H264Decoder>>,
    gh264: (gst::Pipeline, AppSrc),
    channel: (channel::Sender<()>, channel::Receiver<()>),
    stop: UnsafeCell<Option<bool>>,
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

        let h264pipeline = gst::parse_launch(
            "appsrc name=h264-src ! h264parse ! amcviddec-omxgoogleh264decoder ! glimagesink name=videosink sync=false",
        )
        .unwrap();

        let h264pipeline = h264pipeline.dynamic_cast::<gst::Pipeline>().unwrap();

        let mut h264_src = None;

        for elem in h264pipeline.children() {
            // println!("{}", elem.name());
            if elem.name() == "h264-src" {
                h264_src = Some(elem.dynamic_cast::<gst_app::AppSrc>().unwrap());
                break;
            }
        }

        let caps = gst::Caps::from_str(
            "video/x-h264,colorimetry=bt709,stream-format=(string)byte-stream,alignment=(string)au",
        )
        .unwrap();

        let h264_src = h264_src.unwrap();

        h264_src.set_caps(Some(&caps));
        h264_src.set_is_live(true);
        h264_src.set_stream_type(gst_app::AppStreamType::Stream);
        h264_src.set_format(gst::Format::Time);
        h264_src.set_property("emit-signals", true);

        Self {
            alac: (alac_pipeline, alac_appsrc, alac_volume),
            aac_eld: (aac_eld_pipeline, aac_eld_appsrc, aac_eld_volume),
            audio_compression_type: CompressionType::Alac.into(),
            h264_decoder: UnsafeCell::new(None),
            channel: channel::unbounded(),
            gh264: (h264pipeline, h264_src),
            stop: UnsafeCell::new(None),
        }
    }

    pub fn set_surface(&self, surface: &NativeWindow) {
        // self.set_overlay(surface);
        if let Some(h264_decoder) = unsafe { (*self.h264_decoder.get()).as_ref() } {
            if let Err(err) = h264_decoder.set_surface(surface) {
                log::error!("Error setting surface: {:?}", err);
            } else {
                // h264_decoder.start_decode().expect("Error starting decode");
            }
        }
    }

    pub fn set_overlay(&self, surface: &NativeWindow) {
        let elem = 'e: {
            for elem in self.gh264.0.children() {
                if elem.name() == "videosink" {
                    break 'e Some(elem);
                }
            }
            None
        };
        unsafe {
            gst_video_sys::gst_video_overlay_set_window_handle(
                elem.unwrap().as_ptr() as *mut gst_video_sys::GstVideoOverlay,
                // elem.unwrap().as_ptr() as *mut gst_video_sys::GstVideoOverlay,
                surface.ptr().addr().into(),
            );
        }
    }

    pub fn stop(&self) {
        let is_top = unsafe { &mut *self.stop.get() };
        if is_top.is_some() {
            is_top.take();
        } else {
            is_top.replace(true);
            if let Some(h264_decoder) = unsafe { (*self.h264_decoder.get()).take() } {
                let _ = h264_decoder.stop_decode();
            }
        }
    }
}

impl AirPlayConsumer for VideoConsumer {
    fn on_video(&self, bytes: Vec<u8>) {
        // let buffer = gst::Buffer::from_slice(bytes);
        // let _ = self.gh264.1.push_buffer(buffer);
        if let Err(err) = unsafe {
            (*self.h264_decoder.get())
                .as_ref()
                .unwrap()
                .decode_buf(&bytes)
        } {
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
        // log::warn!("等待页面加载...");
        // let _ = self.channel.1.recv();
        // log::warn!("页面加载完成...");
        // self.gh264
        //     .0
        //     .set_state(gst::State::Playing)
        //     .expect("Unable to set the pipeline to the `Playing` state");
        // if let Some(surface) = unsafe { NATIVE_WINDOW.as_ref() } {
        //     if let Err(err) = self.h264_decoder.configure(surface) {
        //         log::error!("Error setting surface: {:?}", err);
        //     }
        // } else {

        // }
        if unsafe { NATIVE_WINDOW.is_none() } {
            let _ = self.channel.1.recv();
        }
        let h264_decoder = H264Decoder::default();
        if let Some(surface) = unsafe { NATIVE_WINDOW.as_ref() } {
            if let Err(err) = h264_decoder.configure(surface) {
                log::error!("Error setting surface: {:?}", err);
            }
        }
        if let Err(err) = h264_decoder.start_decode() {
            log::error!("Error playing: {:?}", err);
        }
        unsafe {
            (*self.h264_decoder.get()).replace(h264_decoder);
        }
    }

    fn on_video_src_disconnect(&self) {
        log::info!("OnVideo Disconnect...");
        // self.gh264
        //     .0
        //     .set_state(gst::State::Null)
        //     .expect("Unable to set the pipeline to the `Playing` state");
        // if let Err(err) = self.h264_decoder.stop_decode() {
        //     log::error!("Error stopping: {:?}", err);
        // }
        self.stop();
        let jvm = unsafe { G_JVM.as_ref().unwrap() };
        let mut env = jvm.attach_current_thread().unwrap();
        if let Some(obj) = unsafe { G_OBJ.as_ref() } {
            let _ = env.call_method(obj, "finish", "()V", &[]);
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
        unsafe {
            *self.audio_compression_type.get() = audio_stream_info.compression_type;
        }
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

    fn is_connected(&self) -> bool {
        unsafe { (*self.stop.get()).take().is_none() }
    }
}

/// # Safety
pub unsafe fn native_surface_init(env: JNIEnv, _class: JClass, obj: JObject, surface: JObject) {
    NATIVE_WINDOW.take();
    let surface = ndk::native_window::NativeWindow::from_surface(env.get_raw(), surface.into_raw());
    NATIVE_WINDOW = surface;
    if G_OBJ.is_some() {
        if let Some(video_consumer) = VIDEO_CONSUMER.as_ref() {
            video_consumer.set_surface(NATIVE_WINDOW.as_ref().unwrap());
        }
    } else {
        G_OBJ = Some(env.new_global_ref(obj).unwrap());
        if let Some(video_consumer) = VIDEO_CONSUMER.as_ref() {
            // video_consumer.set_surface(NATIVE_WINDOW.as_ref().unwrap());
            video_consumer.channel.0.send(()).unwrap();
        }
    }
}

/// # Safety
pub unsafe fn native_surface_finalize(_env: JNIEnv, _class: JClass) {
    if let Some(video_consumer) = VIDEO_CONSUMER.as_ref() {
        video_consumer.stop();
    }
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
