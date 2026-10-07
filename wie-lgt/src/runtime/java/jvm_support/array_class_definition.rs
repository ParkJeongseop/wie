use alloc::{borrow::Cow, boxed::Box, format, string::String};
use core::fmt::{self, Debug, Formatter};

use jvm::{ArrayClassDefinition, ClassDefinition, ClassInstance, JavaType, Jvm, Result as JvmResult};

use wie_core_arm::ArmCore;
use wie_jvm_support::native::array_element_size;
use wie_util::{Result, WieError};

use crate::runtime::java::JavaSvcFunctions;

use super::{JavaArrayClassInstance, JavaClassDefinition};

#[derive(Clone)]
pub struct JavaArrayClassDefinition {
    pub class: JavaClassDefinition,
    core: ArmCore,
}

impl JavaArrayClassDefinition {
    pub async fn new(core: &mut ArmCore, jvm: &Jvm, element_type_name: &str, functions: JavaSvcFunctions) -> Result<Self> {
        let class = JavaClassDefinition::new_array(core, jvm, &format!("[{element_type_name}"), functions).await?;
        Ok(Self { class, core: core.clone() })
    }

    pub fn from_class(class: JavaClassDefinition, core: &ArmCore) -> Self {
        Self { class, core: core.clone() }
    }

    fn element_type_descriptor(&self) -> String {
        let class_name = ClassDefinition::name(&self.class);
        class_name[1..].into()
    }

    pub fn element_size(&self) -> usize {
        array_element_size(&self.element_type())
    }

    pub fn element_type(&self) -> JavaType {
        JavaType::parse(&self.element_type_descriptor())
    }
}

#[async_trait::async_trait]
impl ArrayClassDefinition for JavaArrayClassDefinition {
    fn element_type_name(&self) -> Cow<'_, str> {
        self.element_type_descriptor().into()
    }

    async fn instantiate_array(&self, jvm: &Jvm, length: usize) -> JvmResult<Box<dyn ClassInstance>> {
        let mut instance = JavaArrayClassInstance::new(&mut self.core.clone(), self, length);
        if matches!(instance, Err(WieError::AllocationFailure)) {
            // Titles that never call System.gc() otherwise fill the guest heap with garbage.
            jvm.collect_garbage()?;
            instance = JavaArrayClassInstance::new(&mut self.core.clone(), self, length);
        }
        match instance {
            Ok(instance) => Ok(Box::new(instance)),
            Err(error) => {
                Err(super::error::raise_instantiation_failure(jvm, &format!("array {}[{length}]", self.element_type_descriptor()), error).await)
            }
        }
    }
}

impl Debug for JavaArrayClassDefinition {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("JavaArrayClassDefinition").field("class", &self.class).finish()
    }
}
