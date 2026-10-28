use alloc::vec;

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult, runtime::JavaLangString};
use jvm_class_proto::JavaMethodProto;
use jvm_types::{ClassAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::lang::String;

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class org.kwis.msp.handset.HandsetProperty
pub struct HandsetProperty;

impl HandsetProperty {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msp/handset/HandsetProperty",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new(
                    "getSystemProperty",
                    "(Ljava/lang/String;)Ljava/lang/String;",
                    Self::get_system_property,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "setSystemProperty",
                    "(Ljava/lang/String;Ljava/lang/String;)Z",
                    Self::set_system_property,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
            ],
            fields: vec![],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn get_system_property(jvm: &Jvm, _: &mut WieJvmContext, name: ClassInstanceRef<String>) -> JvmResult<ClassInstanceRef<String>> {
        let name = JavaLangString::to_rust_string(jvm, &name).await?;
        tracing::debug!("org.kwis.msp.handset.HandsetProperty::getSystemProperty({name})");

        // The values a Korean handset answers (wfeature's table). VOLUMELEVEL and VIBRATORLEVEL are the
        // number of steps the hardware has, not a setting: titles divide by them or parse them in
        // startApp (요구르팅, 2006월드올림픽, 드래곤나이트2, 메이플스토리 궁수편, 알바_전설_편의점2 died
        // on Integer.parseInt("")). PHONENUMBER is the four digits MC_knlGetSystemProperty answers
        // (see the note there: four digits or fewer is a handset with no line, and a title divides by
        // or slices the number). MIN stays empty, PHONEMODEL too, since titles compare it against
        // handset names.
        let value = match name.as_ref() {
            "VOLUMELEVEL" => "5",
            "VIBRATORLEVEL" => "0",
            "RSSILEVEL" => "30",
            "MAXRSSILEVEL" => "100",
            "BATTERYLEVEL" => "100",
            "MAXBATTLEVEL" => "100",
            "TIMEZONE" => "GMT+09:00",
            "ESN" => "00000000",
            "PHONENUMBER" => "0100",
            "ANNUN_CALL" | "ANNUN_SMS" | "ANNUN_SILENT" | "ANNUN_ALARM" | "ANNUN_SECURITY" => "0",
            "CURRENTCH" | "AIRPLANE_MODE" | "ROAMING_AREA" | "DS_LOCK" => "0",
            _ => "",
        };

        let result = JavaLangString::from_rust_string(jvm, value).await?;
        Ok(result.into())
    }

    async fn set_system_property(_: &Jvm, _: &mut WieJvmContext, id: ClassInstanceRef<String>, value: ClassInstanceRef<String>) -> JvmResult<bool> {
        tracing::warn!("stub org.kwis.msp.handset.HandsetProperty::setSystemProperty({id:?}, {value:?})");

        Ok(false)
    }
}

#[cfg(test)]
mod test {
    use alloc::boxed::Box;

    use jvm::{ClassInstanceRef, runtime::JavaLangString};
    use rustjava_runtime::classes::java::lang::String;
    use test_utils::run_jvm_test;
    use wie_util::Result;

    use crate::get_protos;

    #[test]
    fn test_set_system_property_returns_false() -> Result<()> {
        run_jvm_test(Box::new([get_protos().into()]), |jvm| async move {
            let id: ClassInstanceRef<String> = JavaLangString::from_rust_string(&jvm, "storage.test").await?.into();
            let value: ClassInstanceRef<String> = JavaLangString::from_rust_string(&jvm, "value").await?.into();
            let result: bool = jvm
                .invoke_static(
                    "org/kwis/msp/handset/HandsetProperty",
                    "setSystemProperty",
                    "(Ljava/lang/String;Ljava/lang/String;)Z",
                    (id, value),
                )
                .await?;

            assert!(!result);
            Ok(())
        })
    }
}
