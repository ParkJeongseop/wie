use alloc::vec;

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{FieldAccessFlags, MethodAccessFlags};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class org.kwis.msp.lwc.ProgressComponent
/// A progress bar running from 0 to a maximum in whole steps. Components are not painted yet, so
/// this keeps the three numbers consistent and nothing more.
pub struct ProgressComponent;

impl ProgressComponent {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msp/lwc/ProgressComponent",
            parent_class: Some("org/kwis/msp/lwc/Component"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "(ZI)V", Self::init, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getMaxValue", "()I", Self::get_max_value, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setMaxValue", "(I)V", Self::set_max_value, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getValue", "()I", Self::get_value, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setValue", "(I)I", Self::set_value, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getStep", "()I", Self::get_step, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setStep", "(I)V", Self::set_step, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setMargin", "(II)V", Self::set_margin, MethodAccessFlags::PUBLIC),
            ],
            fields: vec![
                JavaFieldProto::new("maxValue", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("value", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("step", "I", FieldAccessFlags::PRIVATE),
            ],
            access_flags: Default::default(),
        }
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, interactive: bool, max: i32) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.ProgressComponent::<init>({this:?}, {interactive}, {max})");

        let _: () = jvm.invoke_special(&this, "org/kwis/msp/lwc/Component", "<init>", "()V", ()).await?;

        jvm.put_field(&mut this, "org/kwis/msp/lwc/ProgressComponent", "maxValue", "I", max)
            .await?;
        jvm.put_field(&mut this, "org/kwis/msp/lwc/ProgressComponent", "step", "I", 1).await?;

        Ok(())
    }

    async fn get_max_value(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.lwc.ProgressComponent::getMaxValue({this:?})");

        jvm.get_field(&this, "org/kwis/msp/lwc/ProgressComponent", "maxValue", "I").await
    }

    async fn set_max_value(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, max: i32) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.ProgressComponent::setMaxValue({this:?}, {max})");

        if max <= 0 {
            return Err(jvm.exception("java/lang/IllegalArgumentException", "maximum must be positive").await);
        }

        jvm.put_field(&mut this, "org/kwis/msp/lwc/ProgressComponent", "maxValue", "I", max)
            .await?;
        let value: i32 = jvm.get_field(&this, "org/kwis/msp/lwc/ProgressComponent", "value", "I").await?;
        Self::store_value(jvm, &mut this, value).await?;

        Ok(())
    }

    async fn get_value(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.lwc.ProgressComponent::getValue({this:?})");

        jvm.get_field(&this, "org/kwis/msp/lwc/ProgressComponent", "value", "I").await
    }

    async fn set_value(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, value: i32) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.lwc.ProgressComponent::setValue({this:?}, {value})");

        Self::store_value(jvm, &mut this, value).await
    }

    async fn get_step(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.lwc.ProgressComponent::getStep({this:?})");

        jvm.get_field(&this, "org/kwis/msp/lwc/ProgressComponent", "step", "I").await
    }

    async fn set_step(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, step: i32) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.ProgressComponent::setStep({this:?}, {step})");

        let max: i32 = jvm.get_field(&this, "org/kwis/msp/lwc/ProgressComponent", "maxValue", "I").await?;
        if step <= 0 || step > max {
            return Err(jvm
                .exception("java/lang/IllegalArgumentException", "step must be between 1 and the maximum")
                .await);
        }

        jvm.put_field(&mut this, "org/kwis/msp/lwc/ProgressComponent", "step", "I", step).await?;
        let value: i32 = jvm.get_field(&this, "org/kwis/msp/lwc/ProgressComponent", "value", "I").await?;
        Self::store_value(jvm, &mut this, value).await?;

        Ok(())
    }

    async fn set_margin(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, top: i32, bottom: i32) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.ProgressComponent::setMargin({this:?}, {top}, {bottom})");

        Ok(())
    }

    /// Keeps the value within 0..=maximum and on a whole number of steps.
    async fn store_value(jvm: &Jvm, this: &mut ClassInstanceRef<Self>, value: i32) -> JvmResult<i32> {
        let max: i32 = jvm.get_field(this, "org/kwis/msp/lwc/ProgressComponent", "maxValue", "I").await?;
        let step: i32 = jvm.get_field(this, "org/kwis/msp/lwc/ProgressComponent", "step", "I").await?;

        let value = value.clamp(0, max.max(0));
        let value = if step > 0 { value - value % step } else { value };
        jvm.put_field(this, "org/kwis/msp/lwc/ProgressComponent", "value", "I", value).await?;

        Ok(value)
    }
}
