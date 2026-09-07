#![allow(non_snake_case)]

#[path = "../../../engine/src/network.rs"]
mod network;

mod commands;
mod platform;
mod runtime;
mod store;

use crate::platform::AndroidPlatform;
use crate::runtime::AndroidRuntime;
use crate::store::AndroidStore;
use jni::objects::{JObject, JString};
use jni::sys::jstring;
use jni::JNIEnv;
use quicklan_core::app::App;
use quicklan_core::error::Error;
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::Mutex;

struct NativeState {
    app: App<AndroidStore>,
    platform: AndroidPlatform,
    _tokio: tokio::runtime::Runtime,
}

static STATE: Mutex<Option<NativeState>> = Mutex::new(None);

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_quicklan_android_NativeBridge_initialize(
    mut env: JNIEnv,
    _this: JObject,
    directory: JString,
    platform: JObject,
) -> jstring {
    let result = initialize(&mut env, directory, platform);
    to_jstring(&mut env, result)
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_quicklan_android_NativeBridge_invoke(
    mut env: JNIEnv,
    _this: JObject,
    command: JString,
    args: JString,
) -> jstring {
    let result = invoke(&mut env, command, args);
    to_jstring(&mut env, result)
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_quicklan_android_NativeBridge_shutdown(
    _env: JNIEnv,
    _this: JObject,
) {
    shutdown();
}

fn initialize(env: &mut JNIEnv, directory: JString, platform: JObject) -> String {
    match initialize_inner(env, directory, platform) {
        Ok(value) => envelope_ok(value),
        Err(error) => envelope_err(&error_code(error)),
    }
}

fn initialize_inner(env: &mut JNIEnv, directory: JString, platform: JObject) -> Result<Value, Error> {
    shutdown();
    let directory = PathBuf::from(
        String::from(
            env.get_string(&directory)
                .map_err(|_| Error::UnsafePath)?,
        ),
    );
    if directory.as_os_str().is_empty() {
        return Err(Error::UnsafePath);
    }
    let platform = AndroidPlatform::from_env(env, &platform)?;
    let tokio = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_name("quicklan-android")
        .build()
        .map_err(|_| Error::CoreFailed)?;
    let store = AndroidStore::new(directory, platform.clone())?;
    let runtime = AndroidRuntime::new(platform.clone(), tokio.handle().clone());
    let app = App::new(store)?.with_runtime(Box::new(runtime));
    let mut state = STATE.lock().map_err(|_| Error::CoreFailed)?;
    *state = Some(NativeState {
        _tokio: tokio,
        app,
        platform,
    });
    Ok(Value::Null)
}

fn invoke(env: &mut JNIEnv, command: JString, args: JString) -> String {
    match invoke_inner(env, command, args) {
        Ok(value) => envelope_ok(value),
        Err(code) => envelope_err(&code),
    }
}

fn invoke_inner(
    env: &mut JNIEnv,
    command: JString,
    args: JString,
) -> std::result::Result<Value, String> {
    let command: String = env
        .get_string(&command)
        .map_err(|_| error_code(Error::Unauthorized))?
        .into();
    let args: String = env
        .get_string(&args)
        .map_err(|_| error_code(Error::Unauthorized))?
        .into();
    let mut guard = STATE.lock().map_err(|_| error_code(Error::CoreFailed))?;
    let state = guard.as_mut().ok_or_else(|| error_code(Error::HelperUnavailable))?;
    commands::invoke(&mut state.app, &state.platform, &command, &args)
}

fn shutdown() {
    let mut guard = match STATE.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    if let Some(state) = guard.as_mut() {
        let _ = state.app.disconnect();
    }
}


fn envelope_ok(value: Value) -> String {
    json!({ "ok": true, "value": value }).to_string()
}

fn envelope_err(error: &str) -> String {
    json!({ "ok": false, "error": error }).to_string()
}

fn error_code(error: Error) -> String {
    match serde_json::to_value(error) {
        Ok(Value::String(code)) => code,
        _ => "core_failed".into(),
    }
}

fn to_jstring(env: &mut JNIEnv, json: String) -> jstring {
    env.new_string(json)
        .map(|value| value.into_raw())
        .unwrap_or(std::ptr::null_mut())
}
