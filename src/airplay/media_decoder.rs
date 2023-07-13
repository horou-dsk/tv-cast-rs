use std::{
    ffi::CStr,
    sync::atomic::{AtomicI32, Ordering},
    time::{Duration, Instant},
};

use ndk::{
    media::media_codec::{MediaCodec, MediaCodecDirection, MediaFormat},
    native_window::NativeWindow,
};
// use ndk_sys::{ANativeWindow_getFormat, ANativeWindow_setBuffersGeometry};

use crate::{
    airplay::{G_OBJ, NATIVE_WINDOW},
    G_JVM,
};

use super::exo_golomb::ExpGolombDecoder;

#[allow(dead_code)]
pub struct H264Decoder {
    media_codec: MediaCodec,
    media_format: MediaFormat,
    now: Instant,
    width: AtomicI32,
    height: AtomicI32,
}

unsafe impl Sync for H264Decoder {}
unsafe impl Send for H264Decoder {}

impl Default for H264Decoder {
    fn default() -> Self {
        let video_decoder = MediaCodec::from_decoder_type("video/avc").unwrap();
        log::info!("创建 video/avc 硬件解码器");
        let media_format = MediaFormat::default();
        let mime = unsafe { CStr::from_ptr(ndk_sys::AMEDIAFORMAT_KEY_MIME) };
        media_format.set_str(mime.to_str().unwrap(), "video/avc");
        // media_format.set_str("mime", "video/avc");
        media_format.set_i32("width", 1920);
        media_format.set_i32("height", 1080);
        // video_decoder
        //     .configure(&media_format, None, MediaCodecDirection::Decoder)
        //     .unwrap();
        Self {
            media_codec: video_decoder,
            media_format,
            now: Instant::now(),
            width: AtomicI32::new(0),
            height: AtomicI32::new(0),
        }
    }
}

impl H264Decoder {
    pub fn decode_buf(&self, buf: &[u8]) -> ndk::media::Result<()> {
        if buf[4] & 0x1F == 7 && buf[..4] == [0, 0, 0, 1] {
            self.sps_size_change(buf, |width, height| {
                if let Some(_window) = unsafe { NATIVE_WINDOW.as_ref() } {
                    unsafe {
                        // let format = ANativeWindow_getFormat(window.ptr().as_ptr());
                        // ANativeWindow_setBuffersGeometry(
                        //     window.ptr().as_ptr(),
                        //     width,
                        //     height,
                        //     format,
                        // );
                        let mut env = G_JVM.as_ref().unwrap().attach_current_thread().unwrap();
                        env.call_method(
                            G_OBJ.as_ref().unwrap(),
                            "onResize",
                            "(II)V",
                            &[width.into(), height.into()],
                        )
                        .unwrap();
                    }
                }
                log::warn!("宽高改变... width = {:?} height = {:?}!", width, height,);
            });
        }
        let input_buffer = self
            .media_codec
            .dequeue_input_buffer(Duration::from_millis(16))?;
        if let Some(mut input_buffer) = input_buffer {
            let ibuf = input_buffer.buffer_mut();
            ibuf[..buf.len()].copy_from_slice(buf);
            self.media_codec.queue_input_buffer(
                input_buffer,
                0,
                buf.len(),
                self.now.elapsed().as_micros() as u64,
                0,
            )?;
        }

        while let Some(output_buffer) = self
            .media_codec
            .dequeue_output_buffer(Duration::from_millis(0))?
        {
            let flags = output_buffer.flags();
            if flags == 1 {
                // 宽高改变，部分设备不兼容该方式
            }
            self.media_codec
                .release_output_buffer(output_buffer, true)?;
            // if flags as i32 & ndk_sys::AMEDIACODEC_BUFFER_FLAG_END_OF_STREAM == 0 {
            //     // log::info!("end_of_stream");
            //     break;
            // }
        }
        Ok(())
    }

    pub fn start_decode(&self) -> ndk::media::Result<()> {
        self.media_codec.start()
    }

    pub fn stop_decode(&self) -> ndk::media::Result<()> {
        self.width.store(0, Ordering::Relaxed);
        self.height.store(0, Ordering::Relaxed);
        self.media_codec.stop()
    }

    #[allow(dead_code)]
    pub fn set_surface(&self, surface: &NativeWindow) -> ndk::media::Result<()> {
        self.media_codec.set_output_surface(surface)
    }

    #[inline]
    pub fn configure(&self, surface: &NativeWindow) -> ndk::media::Result<()> {
        self.media_codec.configure(
            &self.media_format,
            Some(surface),
            MediaCodecDirection::Decoder,
        )
    }

    #[allow(unused_variables)]
    pub fn sps_size_change<F: Fn(i32, i32)>(&self, sps: &[u8], on_change: F) {
        let profile_idc = sps[5];

        let mut g = ExpGolombDecoder::new(&sps[8..], 0).unwrap();

        let seq_parameter_set_id = g.next_unsigned().unwrap();

        if profile_idc == 100
            || profile_idc == 110
            || profile_idc == 122
            || profile_idc == 244
            || profile_idc == 44
            || profile_idc == 83
            || profile_idc == 86
            || profile_idc == 118
            || profile_idc == 128
            || profile_idc == 138
            || profile_idc == 139
            || profile_idc == 134
            || profile_idc == 135
        {
            let chroma_format_idc = g.next_unsigned().unwrap();
            if chroma_format_idc == 3 {
                let separate_colour_plane_flag = g.next_bit().unwrap();
            }
            let bit_depth_luma_minus8 = g.next_unsigned().unwrap();
            let bit_depth_chroma_minus8 = g.next_unsigned().unwrap();
            let qpprime_y_zero_transform_bypass_flag = g.next_bit().unwrap();
            let seq_scaling_matrix_present_flag = g.next_bit().unwrap();
            if seq_scaling_matrix_present_flag != 0 {
                // let mut seq_scaling_list_present_flag = vec![];
                let len = if chroma_format_idc != 3 { 8 } else { 12 };
                for i in 0..len {
                    let _ = g.next_bit();
                    // seq_scaling_list_present_flag.push(g.next_bit().unwrap());
                    // if seq_scaling_list_present_flag[i] != 0 {
                    // if i < 6 {
                    // } else {
                    // }
                    // }
                }
            }
            let log2_max_frame_num_minus4 = g.next_unsigned().unwrap();
            let pic_order_cnt_type = g.next_unsigned().unwrap();
            if pic_order_cnt_type == 0 {
                let log2_max_pic_order_cnt_lsb_minus4 = g.next_unsigned().unwrap();
            } else if pic_order_cnt_type == 1 {
                let delta_pic_order_always_zero_flag = g.next_bit().unwrap();
                let offset_for_non_ref_pic = g.next_signed().unwrap();
                let offset_for_top_to_bottom_field = g.next_signed().unwrap();
                let num_ref_frames_in_pic_order_cnt_cycle = g.next_unsigned().unwrap();
                // let mut offset_for_ref_frame = vec![];
                for _ in 0..num_ref_frames_in_pic_order_cnt_cycle {
                    let _ = g.next_signed();
                    // offset_for_ref_frame.push(g.next_signed().unwrap());
                }
            }
            let max_num_ref_frames = g.next_unsigned().unwrap();
            let gaps_in_frame_num_value_allowed_flag = g.next_bit().unwrap();
            let pic_width_in_mbs_minus1 = g.next_unsigned().unwrap();
            let pic_height_in_map_units_minus1 = g.next_unsigned().unwrap();
            let (width, height) = (
                ((pic_width_in_mbs_minus1 + 1) * 16) as i32,
                ((pic_height_in_map_units_minus1 + 1) * 16) as i32,
            );
            if self.width.load(Ordering::Relaxed) != width
                || self.height.load(Ordering::Relaxed) != height
            {
                self.width.store(width, Ordering::Relaxed);
                self.height.store(height, Ordering::Relaxed);
                on_change(width, height);
            }
        }
    }
}
