use alloc::boxed::Box;

use jvm::ClassInstance;

use wie_core_arm::ArmCore;
use wie_jvm_support::native::NativeJavaValueCodec;

use super::{array_class_instance::JavaArrayClassInstance, class_instance::JavaClassInstance};

#[derive(Clone)]
pub struct JavaValueCodec {
    core: ArmCore,
}

impl JavaValueCodec {
    pub fn new(core: &ArmCore) -> Self {
        Self { core: core.clone() }
    }
}

impl NativeJavaValueCodec for JavaValueCodec {
    fn object_from_raw(&self, raw: u32) -> Box<dyn ClassInstance> {
        let instance = JavaClassInstance::from_raw(raw, &self.core);
        // Apps sometimes pass garbage where an object is declared (e.g. a String to
        // String.<init>([CII)V in a rarely-taken error path). Reading its metadata
        // then fabricates array elements and panics; fall back to a plain instance
        // so the JVM surfaces a catchable error like real hardware would.
        match instance.class().and_then(|x| x.name()) {
            Ok(name) if name.starts_with('[') => Box::new(JavaArrayClassInstance::from_raw(raw, &self.core)),
            Ok(_) => Box::new(instance),
            Err(e) => {
                tracing::warn!("invalid object pointer {raw:#x} passed as java value: {e:?}");
                Box::new(instance)
            }
        }
    }

    fn object_to_raw(&self, object: &dyn ClassInstance) -> u32 {
        if let Some(instance) = object.as_any().downcast_ref::<JavaClassInstance>() {
            instance.ptr_raw
        } else {
            object.as_any().downcast_ref::<JavaArrayClassInstance>().unwrap().class_instance.ptr_raw
        }
    }
}
