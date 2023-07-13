#![feature(maybe_uninit_slice)]
#![feature(lazy_cell)]
#![feature(result_option_inspect)]
#![feature(strict_provenance)]

use std::{ffi::c_void, path::Path};

use android_logger::Config;
use jni::{
    objects::{GlobalRef, JObject, JString},
    sys::jint,
    JNIEnv, JavaVM, NativeMethod,
};
use log::LevelFilter;

use crate::{
    actions::jni_action::AVTransportAction,
    airplay::{native_surface_finalize, native_surface_init},
};

pub mod constant;
pub mod header;
pub mod protocol;
pub mod setting;
pub mod ssdp;

pub mod actions;
pub mod airplay;
pub mod android;
mod entry;
pub mod net;
pub mod routers;

static mut G_JVM: Option<JavaVM> = None;
static mut G_SERVICE: Option<GlobalRef> = None;

#[no_mangle]
pub extern "C" fn Java_com_ycsoft_smartbox_services_TPServices_rustMethod(
    mut env: JNIEnv,
    service: JObject,
    input: JString,
    path: JString,
    obj: JObject<'static>,
) {
    unsafe {
        G_SERVICE = Some(env.new_global_ref(service).unwrap());
    }
    let global_ref = env.new_global_ref(obj).unwrap();
    let input: String = env.get_string(&input).unwrap().into();
    let path: String = env.get_string(&path).unwrap().into();
    let av_action = AVTransportAction::new(env.get_java_vm().unwrap(), global_ref);
    log::info!("开始运行服务...................");
    std::thread::spawn(|| {
        actix_web::rt::System::new().block_on(async move {
            let name = input.clone();
            // let (_, _) = tokio::join!(
            //     airplay::airplay_run(name),
            //     entry::dlna_run(input, Path::new(&path), av_action)
            // );
            tokio::task::spawn(async {
                if let Err(err) = airplay::airplay_run(name).await {
                    log::error!("run airplay error {err:?}");
                }
            });
            if let Err(err) = entry::dlna_run(input, Path::new(&path), av_action).await {
                log::error!("run main error {err:?}");
            }
        });
        log::error!("异常结束...!!!!!!!!!!!!!!!!!!!!!!!!!!");
    });
}

#[allow(non_snake_case)]
#[no_mangle]
fn JNI_OnLoad(jvm: JavaVM, _reserved: *mut c_void) -> jint {
    // std::env::set_var("GST_DEBUG", "5");
    android_logger::init_once(Config::default().with_max_level(LevelFilter::Info).format(
        |f, record| {
            write!(
                f,
                "[{}:{}] {}",
                record.module_path().unwrap_or("unknown"),
                record.line().unwrap_or(0),
                record.args()
            )
        },
    ));
    log_panics::init();
    let mut env = match jvm.get_env() {
        Ok(env) => env,
        Err(err) => {
            log::error!("Could not retrieve JNIEnv, error: {}", err);
            return 0;
        }
    };

    let version: jint = match env.get_version() {
        Ok(v) => v.into(),
        Err(err) => {
            log::error!("Could not retrieve JNI version, error: {}", err);
            return 0;
        }
    };
    let kclass = env
        .find_class("com/ycsoft/smartbox/services/TPServices")
        .unwrap();
    let methods = [
        NativeMethod {
            name: "nativeSurfaceInit".into(),
            sig: "(Lcom/ycsoft/smartbox/tvcast/AirplayActivity;Landroid/view/Surface;)V".into(),
            fn_ptr: native_surface_init as *mut c_void,
        },
        NativeMethod {
            name: "nativeSurfaceFinalize".into(),
            sig: "()V".into(),
            fn_ptr: native_surface_finalize as *mut c_void,
        },
    ];
    env.register_native_methods(kclass, &methods).unwrap();
    log::info!("JNI OnLoad in Rust...........");
    unsafe {
        G_JVM = Some(jvm);
    }
    version
}
