use alloc::{
    boxed::Box,
    collections::{BTreeMap, BTreeSet},
    string::ToString,
    sync::Arc,
    vec::Vec,
};
use core::{ops::Deref, ops::DerefMut, pin::Pin};

use jvm::{ClassDefinition, ClassInstance, ExternalRoots, Jvm, Result as JvmResult};
use jvm_class_proto::JavaClassProto;
use spin::Mutex;

use wie_core_arm::ArmCore;
use wie_jvm_support::JvmImplementation;
use wie_util::{ByteRead, Result};

use crate::runtime::java::{JavaSvcFunctions, register_java_svc_handler};

use super::{JavaArrayClassDefinition, JavaClassDefinition};

#[derive(Clone)]
pub struct LgtJvmImplementation {
    core: ArmCore,
    functions: JavaSvcFunctions,
}

impl LgtJvmImplementation {
    pub fn new(core: &mut ArmCore) -> Result<Self> {
        let functions = Arc::new(Mutex::new(BTreeMap::new()));
        register_java_svc_handler(core, &functions)?;

        Ok(Self {
            core: core.clone(),
            functions,
        })
    }
}

impl JvmImplementation for LgtJvmImplementation {
    fn external_roots(&self) -> Option<Box<dyn ExternalRoots>> {
        Some(Box::new(self.clone()))
    }

    fn define_class_rust<'a, C, Context>(
        &'a self,
        jvm: &'a Jvm,
        proto: JavaClassProto<C>,
        context: Context,
    ) -> Pin<Box<dyn Future<Output = JvmResult<Box<dyn ClassDefinition>>> + Send + 'a>>
    where
        C: ?Sized + 'static + Send,
        Context: Sync + Send + DerefMut + Deref<Target = C> + Clone + 'static,
    {
        Box::pin(async move {
            match JavaClassDefinition::new(&mut self.core.clone(), jvm, proto, context, self.functions.clone()).await {
                Ok(class) => Ok(Box::new(class) as Box<_>),
                Err(error) => Err(super::error::raise(jvm, "net/wie/WieError", &error.to_string()).await),
            }
        })
    }

    async fn define_array_class(&self, jvm: &Jvm, element_type_name: &str) -> JvmResult<Box<dyn ClassDefinition>> {
        match JavaArrayClassDefinition::new(&mut self.core.clone(), jvm, element_type_name, self.functions.clone()).await {
            Ok(class) => Ok(Box::new(class)),
            Err(error) => Err(super::error::raise(jvm, "net/wie/WieError", &error.to_string()).await),
        }
    }
}

/// AOT-compiled Java keeps object references in ARM registers and on ARM stacks, where the collector
/// cannot see them; without this, `System.gc()` frees objects that are only held by a running guest
/// method. Whole thread stacks are scanned rather than the part above each saved stack pointer,
/// because a task suspended inside guest code keeps its frames where the engine's current stack
/// pointer does not point (the live stack pointer was observed at another thread's stack top while
/// that thread's frames, holding the only reference to a StringBuffer, lay below it). Stale words in
/// dead frames keep a few garbage objects alive; that is the price.
impl ExternalRoots for LgtJvmImplementation {
    fn roots(&self, objects: &[&dyn ClassInstance]) -> anyhow::Result<Vec<usize>> {
        const CHUNK: usize = 64 * 1024;

        // The live registers and the frames above each saved stack pointer; the whole stacks follow.
        let mut words = self.core.stack_words().unwrap_or_default().into_iter().collect::<BTreeSet<_>>();
        let mut chunk = alloc::vec![0u8; CHUNK];
        for (base, end) in self.core.thread_stacks() {
            let mut address = base & !3;
            while address < end {
                let length = ((end - address) as usize).min(CHUNK);
                if self.core.read_bytes(address, &mut chunk[..length]).is_err() {
                    break;
                }
                words.extend(chunk[..length].as_chunks::<4>().0.iter().map(|word| u32::from_le_bytes(*word)));
                address += length as u32;
            }
        }

        Ok(objects
            .iter()
            .map(|object| object.identity())
            .filter(|identity| words.contains(&(*identity as u32)))
            .collect())
    }
}
