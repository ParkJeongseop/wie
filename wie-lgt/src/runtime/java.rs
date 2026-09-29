use alloc::{boxed::Box, collections::BTreeMap, format, string::String, sync::Arc};

use spin::Mutex;
use wipi_types::lgt::java::{
    LgtJavaClass as RawJavaClass, LgtJavaClassDescriptor as RawJavaClassDescriptor, LgtJavaClassInstance as RawJavaClassInstance,
    LgtJavaClassMethod as RawJavaMethod,
};

use wie_core_arm::{ArmCore, JumpTo, RegisteredFunction, SvcId};
use wie_util::{Result, WieError, read_generic, read_null_terminated_string_bytes};

use crate::runtime::{SVC_CATEGORY_JAVA, SVC_CATEGORY_MISSING_JAVA_VTABLE_ENTRY};

mod abi;
pub mod classes;
mod exception;
mod interface;
mod jvm_support;

pub use interface::{get_java_interface_method, register_java_system_svc_handler};
pub use jvm_support::LgtJvmSupport;

pub type JavaSvcFunctions = Arc<Mutex<BTreeMap<u32, Arc<Box<dyn RegisteredFunction>>>>>;

async fn handle_java_svc(core: &mut ArmCore, functions: &mut JavaSvcFunctions, id: SvcId) -> Result<JumpTo> {
    let (_, lr) = core.read_pc_lr()?;
    if tracing::enabled!(tracing::Level::TRACE) {
        tracing::trace!("java svc {}", describe_java_svc(core, id.0));
    }
    let function = functions
        .lock()
        .get(&id.0)
        .cloned()
        .ok_or_else(|| WieError::FatalError(alloc::format!("Unknown LGT Java SVC id {:#x}", id.0)))?;

    match function.call(core).await {
        Ok(()) => Ok(JumpTo(lr)),
        Err(WieError::JavaException(ptr_exception)) => match exception::unwind(core, ptr_exception)? {
            Some(resume_address) => Ok(JumpTo(resume_address)),
            None => Err(WieError::JavaException(ptr_exception)),
        },
        Err(error) => {
            tracing::error!("LGT Java native {} failed: {error}", describe_java_svc(core, id.0));

            Err(error)
        }
    }
}

/// Names the guest method a Java svc id was registered for, or falls back to the raw id when the
/// registration is not backed by a readable `RawJavaMethod` (array class helpers, corrupted memory).
fn describe_java_svc(core: &ArmCore, id: u32) -> String {
    let read = |ptr: u32| -> Option<String> {
        if ptr == 0 {
            return None;
        }
        let bytes = read_null_terminated_string_bytes(core, ptr).ok()?;
        String::from_utf8(bytes).ok()
    };
    let described = read_generic::<RawJavaMethod, _>(core, id)
        .ok()
        .and_then(|raw| Some(format!("{}{} (id {id:#x})", read(raw.ptr_name)?, read(raw.ptr_descriptor)?)));
    described.unwrap_or_else(|| format!("id {id:#x}"))
}

/// Best-effort description of a guest pointer as a Java object: its class name and how many
/// method slots its vtable has. Used when a call through a vtable faults, so the log names the
/// receiver whose slot was missing instead of only the pc.
pub(crate) fn describe_guest_object(core: &ArmCore, ptr_instance: u32) -> String {
    let describe = || -> Option<String> {
        if ptr_instance == 0 {
            return None;
        }
        let instance: RawJavaClassInstance = read_generic(core, ptr_instance).ok()?;
        let ptr_class: u32 = read_generic(core, instance.ptr_dispatch_table).ok()?;
        let class: RawJavaClass = read_generic(core, ptr_class).ok()?;
        let descriptor: RawJavaClassDescriptor = read_generic(core, class.ptr_descriptor).ok()?;
        let name = String::from_utf8(read_null_terminated_string_bytes(core, descriptor.ptr_name).ok()?).ok()?;
        Some(format!("{ptr_instance:#x} ({name}, {} vtable slots)", descriptor.vtable_count))
    };
    describe().unwrap_or_else(|| format!("{ptr_instance:#x} (not a Java object)"))
}

async fn handle_missing_java_vtable_entry(core: &mut ArmCore, _: &mut (), id: SvcId) -> Result<JumpTo> {
    let ptr_instance = core.read_param(0)?;
    let instance: RawJavaClassInstance = read_generic(core, ptr_instance)?;
    let ptr_class: u32 = read_generic(core, instance.ptr_dispatch_table)?;
    let class: RawJavaClass = read_generic(core, ptr_class)?;
    let descriptor: RawJavaClassDescriptor = read_generic(core, class.ptr_descriptor)?;
    let class_name = String::from_utf8(read_null_terminated_string_bytes(core, descriptor.ptr_name)?)
        .map_err(|error| WieError::FatalError(format!("Invalid LGT class name: {error}")))?;

    Err(WieError::Unimplemented(format!("{class_name} vtable index {}", id.0)))
}

pub fn register_java_svc_handler(core: &mut ArmCore, functions: &JavaSvcFunctions) -> Result<()> {
    core.register_svc_handler(SVC_CATEGORY_JAVA, handle_java_svc, functions)?;
    core.register_svc_handler(SVC_CATEGORY_MISSING_JAVA_VTABLE_ENTRY, handle_missing_java_vtable_entry, &())
}
