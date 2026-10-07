use alloc::{string::ToString, vec};

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult, runtime::JavaLangString};
use jvm_class_proto::JavaMethodProto;
use jvm_types::{ClassAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::lang::String;

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class mmpp.phone.Phone
pub struct Phone;

impl Phone {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "mmpp/phone/Phone",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "getProperty",
                    "(Ljava/lang/String;)Ljava/lang/String;",
                    Self::get_property,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "placeCall",
                    "(Ljava/lang/String;)V",
                    Self::place_call,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "invokeWAPBrowser",
                    "(Ljava/lang/String;)V",
                    Self::invoke_wap_browser,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "finalizeNetwork",
                    "()V",
                    Self::finalize_network,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
            ],
            fields: vec![],
            access_flags: ClassAccessFlags::PUBLIC | ClassAccessFlags::FINAL,
        }
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("mmpp.phone.Phone::<init>({this:?})");

        jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await
    }

    async fn get_property(jvm: &Jvm, context: &mut WieJvmContext, key: ClassInstanceRef<String>) -> JvmResult<ClassInstanceRef<String>> {
        let key = JavaLangString::to_rust_string(jvm, &key).await?;

        let value = match key.as_str() {
            // the subscriber number; the same placeholder the SK-VM platform reports
            "MIN" => Some("01000000000".to_string()),
            "cdma.callStatus" => Some("false".to_string()),
            "phone.RSSILevel" => Some("4".to_string()),
            "phone.batteryLevel" => Some("4".to_string()),
            "phone.LCD.colorDepth" => Some("16".to_string()),
            "phone.LCD.width" => Some(context.system().platform().screen().width().to_string()),
            "phone.LCD.height" => Some(context.system().platform().screen().height().to_string()),
            _ => None,
        };
        tracing::debug!("mmpp.phone.Phone::getProperty({key}) = {value:?}");

        match value {
            Some(value) => Ok(JavaLangString::from_rust_string(jvm, &value).await?.into()),
            None => Ok(None.into()),
        }
    }

    async fn place_call(jvm: &Jvm, _: &mut WieJvmContext, phone_number: ClassInstanceRef<String>) -> JvmResult<()> {
        let phone_number = JavaLangString::to_rust_string(jvm, &phone_number).await?;
        tracing::warn!("stub mmpp.phone.Phone::placeCall({phone_number})");

        Ok(())
    }

    async fn invoke_wap_browser(jvm: &Jvm, _: &mut WieJvmContext, url: ClassInstanceRef<String>) -> JvmResult<()> {
        let url = JavaLangString::to_rust_string(jvm, &url).await?;
        tracing::warn!("stub mmpp.phone.Phone::invokeWAPBrowser({url})");

        Ok(())
    }

    // Not in the published API; titles call it after a network session. There is no network to release.
    async fn finalize_network(_: &Jvm, _: &mut WieJvmContext) -> JvmResult<()> {
        tracing::debug!("mmpp.phone.Phone::finalizeNetwork()");

        Ok(())
    }
}
