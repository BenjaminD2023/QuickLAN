use crate::network::MobileTun;
use jni::objects::{GlobalRef, JObject, JString, JValue};
use jni::JNIEnv;
use jni::JavaVM;
use quicklan_core::error::{Error, Result};
use std::sync::Arc;

#[derive(Clone)]
pub struct AndroidPlatform {
    vm: Arc<JavaVM>,
    obj: GlobalRef,
}

impl AndroidPlatform {
    pub fn from_env(env: &JNIEnv, platform: &JObject) -> Result<Self> {
        if platform.is_null() {
            return Err(Error::HelperUnavailable);
        }
        let vm = env.get_java_vm().map_err(|_| Error::CoreFailed)?;
        let obj = env
            .new_global_ref(platform)
            .map_err(|_| Error::CoreFailed)?;
        Ok(Self { vm: Arc::new(vm), obj })
    }

    fn with_env<T>(&self, f: impl FnOnce(&mut JNIEnv) -> Result<T>) -> Result<T> {
        let mut env = self
            .vm
            .attach_current_thread()
            .map_err(|_| Error::CoreFailed)?;
        let result = f(&mut env);
        if env.exception_check().unwrap_or(false) {
            let _ = env.exception_clear();
            return Err(Error::CoreFailed);
        }
        result
    }

    pub fn read_secret(&self, id: &str) -> Result<String> {
        self.with_env(|env| {
            let j_id = env.new_string(id).map_err(|_| Error::StorageUnavailable)?;
            let value = env
                .call_method(
                    &self.obj,
                    "readSecret",
                    "(Ljava/lang/String;)Ljava/lang/String;",
                    &[JValue::Object(&j_id)],
                )
                .map_err(|_| Error::StorageUnavailable)?;
            let obj = value.l().map_err(|_| Error::StorageUnavailable)?;
            if obj.is_null() {
                return Err(Error::StorageUnavailable);
            }
            let text: String = env
                .get_string(&JString::from(obj))
                .map_err(|_| Error::StorageUnavailable)?
                .into();
            Ok(text)
        })
    }

    pub fn write_secret(&self, id: &str, value: &str) -> Result<()> {
        self.with_env(|env| {
            let j_id = env.new_string(id).map_err(|_| Error::StorageUnavailable)?;
            let j_value = env
                .new_string(value)
                .map_err(|_| Error::StorageUnavailable)?;
            let ok = env
                .call_method(
                    &self.obj,
                    "writeSecret",
                    "(Ljava/lang/String;Ljava/lang/String;)Z",
                    &[JValue::Object(&j_id), JValue::Object(&j_value)],
                )
                .map_err(|_| Error::StorageUnavailable)?
                .z()
                .map_err(|_| Error::StorageUnavailable)?;
            if ok {
                Ok(())
            } else {
                Err(Error::StorageUnavailable)
            }
        })
    }

    pub fn remove_secret(&self, id: &str) -> Result<()> {
        self.with_env(|env| {
            let j_id = env.new_string(id).map_err(|_| Error::StorageUnavailable)?;
            let ok = env
                .call_method(
                    &self.obj,
                    "removeSecret",
                    "(Ljava/lang/String;)Z",
                    &[JValue::Object(&j_id)],
                )
                .map_err(|_| Error::StorageUnavailable)?
                .z()
                .map_err(|_| Error::StorageUnavailable)?;
            if ok {
                Ok(())
            } else {
                Err(Error::StorageUnavailable)
            }
        })
    }

    pub fn copy_text(&self, text: &str, sensitive: bool) -> Result<()> {
        self.with_env(|env| {
            let j_text = env.new_string(text).map_err(|_| Error::StorageUnavailable)?;
            let ok = env
                .call_method(
                    &self.obj,
                    "copyText",
                    "(Ljava/lang/String;Z)Z",
                    &[JValue::Object(&j_text), JValue::Bool(u8::from(sensitive))],
                )
                .map_err(|_| Error::StorageUnavailable)?
                .z()
                .map_err(|_| Error::StorageUnavailable)?;
            if ok {
                Ok(())
            } else {
                Err(Error::StorageUnavailable)
            }
        })
    }

    pub fn local_endpoints(&self) -> Result<serde_json::Value> {
        self.with_env(|env| {
            let value = env
                .call_method(&self.obj, "localEndpoints", "()Ljava/lang/String;", &[])
                .map_err(|_| Error::CoreFailed)?;
            let obj = value.l().map_err(|_| Error::CoreFailed)?;
            if obj.is_null() {
                return Err(Error::CoreFailed);
            }
            let text: String = env
                .get_string(&JString::from(obj))
                .map_err(|_| Error::CoreFailed)?
                .into();
            let parsed: serde_json::Value =
                serde_json::from_str(&text).map_err(|_| Error::CoreFailed)?;
            Ok(parsed)
        })
    }
}

impl MobileTun for AndroidPlatform {
    fn establish(&self, ip: &str, subnet: &str) -> Result<i32> {
        self.with_env(|env| {
            let j_ip = env.new_string(ip).map_err(|_| Error::CoreFailed)?;
            let j_subnet = env.new_string(subnet).map_err(|_| Error::CoreFailed)?;
            env.call_method(
                &self.obj,
                "establishTun",
                "(Ljava/lang/String;Ljava/lang/String;)I",
                &[JValue::Object(&j_ip), JValue::Object(&j_subnet)],
            )
            .map_err(|_| Error::CoreFailed)?
            .i()
            .map_err(|_| Error::CoreFailed)
            .and_then(|fd| match fd {
                -2 => Err(Error::RouteConflict),
                -3 => Err(Error::PermissionDenied),
                fd if fd >= 0 => Ok(fd),
                _ => Err(Error::CoreFailed),
            })
        })
    }

    fn close(&self) {
        let _ = self.with_env(|env| {
            env.call_method(&self.obj, "closeTun", "()V", &[])
                .map_err(|_| Error::CoreFailed)?;
            Ok(())
        });
    }
}
