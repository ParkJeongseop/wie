use alloc::boxed::Box;

use jvm::{ClassDefinition, ClassInstance};

use wie_core_arm::ArmCore;
use wie_jvm_support::native::NativeJavaValueCodec;

use super::{JavaArrayClassInstance, JavaClassInstance};

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
        let class = instance.class().unwrap_or_else(|error| {
            // A reference whose header no longer reads is almost always one the collector already
            // freed; name the pointer so the matching "Destroying" trace line can be found.
            panic!("LGT object reference {raw:#x} has no readable class header: {error}")
        });
        if ClassDefinition::name(&class).starts_with('[') {
            Box::new(JavaArrayClassInstance::from_raw(raw, &self.core))
        } else {
            Box::new(instance)
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
