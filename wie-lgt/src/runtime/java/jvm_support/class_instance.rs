use alloc::{boxed::Box, vec};
use core::{
    fmt::{self, Debug, Formatter},
    hash::{Hash, Hasher},
    mem::size_of,
};

use jvm::{ClassDefinition, ClassInstance, Field, JavaType, JavaValue, Result as JvmResult};
use jvm_types::FieldAccessFlags;
use wipi_types::lgt::java::{
    LgtJavaClass as RawJavaClass, LgtJavaClassDescriptor as RawJavaClassDescriptor, LgtJavaClassInstance as RawJavaClassInstance,
};

use wie_core_arm::{Allocator, ArmCore};
use wie_jvm_support::native::NativeJavaValueCodec;
use wie_util::{ByteRead, ByteWrite, Result, read_generic, read_null_terminated_string_bytes, write_generic};

use super::{JavaClassDefinition, JavaField, JavaReferenceField, LgtJvmWord, value::JavaValueCodec};

#[derive(Clone)]
pub struct JavaClassInstance {
    pub ptr_raw: u32,
    core: ArmCore,
}

impl JavaClassInstance {
    pub fn from_raw(ptr_raw: u32, core: &ArmCore) -> Self {
        Self { ptr_raw, core: core.clone() }
    }

    pub fn new(core: &mut ArmCore, class: &JavaClassDefinition) -> Result<Self> {
        Self::instantiate(core, class, class.instance_field_word_count()? * size_of::<LgtJvmWord>())
    }

    pub fn instantiate(core: &mut ArmCore, class: &JavaClassDefinition, storage_size: usize) -> Result<Self> {
        let ptr_raw = Allocator::alloc(core, size_of::<RawJavaClassInstance>() as u32)?;
        let allocated_storage_size = storage_size.max(size_of::<LgtJvmWord>());
        let ptr_fields = Allocator::alloc(core, allocated_storage_size as u32)?;
        core.write_bytes(ptr_fields, &vec![0; allocated_storage_size])?;

        let extension_size = class.extension_word_count()? * size_of::<LgtJvmWord>();
        let ptr_extension = if extension_size == 0 {
            0
        } else {
            let ptr_extension = Allocator::alloc(core, extension_size as u32)?;
            core.write_bytes(ptr_extension, &vec![0; extension_size])?;
            ptr_extension
        };

        write_generic(
            core,
            ptr_raw,
            RawJavaClassInstance {
                ptr_dispatch_table: class.ptr_vtable()?,
                unk1: ptr_extension,
                ptr_fields,
            },
        )?;

        Ok(Self::from_raw(ptr_raw, core))
    }

    /// The extension block holding wie-defined instance fields (header `unk1`, which the LGT ABI
    /// leaves at zero). Allocated on demand for instances that predate their class's fields.
    pub fn extension_address(&self, class: &JavaClassDefinition) -> Result<u32> {
        let mut raw: RawJavaClassInstance = read_generic(&self.core, self.ptr_raw)?;
        if raw.unk1 == 0 {
            let size = (class.extension_word_count()? * size_of::<LgtJvmWord>()).max(size_of::<LgtJvmWord>());
            let mut core = self.core.clone();
            raw.unk1 = Allocator::alloc(&mut core, size as u32)?;
            core.write_bytes(raw.unk1, &vec![0; size])?;
            write_generic(&mut core, self.ptr_raw, raw)?;
        }

        Ok(raw.unk1)
    }

    fn java_field_address(&self, field: &JavaField) -> Result<u32> {
        let word_index = field.word_index()?;
        if field.is_extension()? {
            Ok(self.extension_address(&self.class()?)? + word_index * size_of::<LgtJvmWord>() as u32)
        } else {
            self.field_address(word_index)
        }
    }

    /// Whether `ptr_instance` points at an allocated object whose header chain is intact
    /// (instance -> dispatch table -> class record whose vtable pointer is that dispatch table).
    pub fn is_live_instance(core: &ArmCore, ptr_instance: u32) -> bool {
        if ptr_instance == 0 || !Allocator::is_allocated(core, ptr_instance, size_of::<RawJavaClassInstance>() as u32).unwrap_or(false) {
            return false;
        }
        let Ok(instance): Result<RawJavaClassInstance> = read_generic(core, ptr_instance) else {
            return false;
        };
        let Ok(ptr_class): Result<u32> = read_generic(core, instance.ptr_dispatch_table) else {
            return false;
        };
        let Ok(class): Result<RawJavaClass> = read_generic(core, ptr_class) else {
            return false;
        };
        if class.unk1 == instance.ptr_dispatch_table {
            return true;
        }
        // An instance born before its class's vtable was replaced still points at the old table;
        // accept it when the class record it names reads as a class (descriptor and name).
        let Ok(descriptor): Result<RawJavaClassDescriptor> = read_generic(core, class.ptr_descriptor) else {
            return false;
        };
        descriptor.ptr_name != 0
            && read_null_terminated_string_bytes(core, descriptor.ptr_name)
                .is_ok_and(|name| !name.is_empty() && name.len() < 256 && name.iter().all(|byte| byte.is_ascii_graphic()))
    }

    pub fn destroy_with_storage(mut self, storage_size: usize) -> Result<()> {
        let raw: RawJavaClassInstance = read_generic(&self.core, self.ptr_raw)?;
        if raw.unk1 != 0 {
            let extension_size = (self.class()?.extension_word_count()? * size_of::<LgtJvmWord>()).max(size_of::<LgtJvmWord>());
            Allocator::free(&mut self.core, raw.unk1, extension_size as u32)?;
        }
        Allocator::free(&mut self.core, raw.ptr_fields, storage_size.max(size_of::<LgtJvmWord>()) as u32)?;
        Allocator::free(&mut self.core, self.ptr_raw, size_of::<RawJavaClassInstance>() as u32)
    }

    pub fn class(&self) -> Result<JavaClassDefinition> {
        let raw: RawJavaClassInstance = read_generic(&self.core, self.ptr_raw)?;
        let ptr_class = read_generic(&self.core, raw.ptr_dispatch_table)?;
        Ok(JavaClassDefinition::from_raw(ptr_class, &self.core))
    }

    pub fn ptr_fields(&self) -> Result<u32> {
        let raw: RawJavaClassInstance = read_generic(&self.core, self.ptr_raw)?;
        Ok(raw.ptr_fields)
    }

    pub fn storage_address(&self, byte_offset: usize) -> Result<u32> {
        Ok(self.ptr_fields()? + byte_offset as u32)
    }

    pub fn storage_size(&self) -> Result<usize> {
        Ok(self.class()?.instance_field_word_count()? * size_of::<LgtJvmWord>())
    }

    fn field_address(&self, word_index: u32) -> Result<u32> {
        self.storage_address(word_index as usize * size_of::<LgtJvmWord>())
    }
}

#[async_trait::async_trait]
impl ClassInstance for JavaClassInstance {
    fn destroy(self: Box<Self>) {
        // The collector can only free what it can size. An object whose header was overwritten by
        // guest code is leaked instead of taking the whole emulator down mid-collection.
        if !Self::is_live_instance(&self.core, self.ptr_raw) {
            tracing::error!("Not freeing LGT object {:#x}: its class header no longer reads", self.ptr_raw);
            return;
        }
        let storage_size = self.storage_size().unwrap();
        (*self).destroy_with_storage(storage_size).unwrap();
    }

    fn identity(&self) -> usize {
        self.ptr_raw as usize
    }

    fn shallow_clone(&self) -> JvmResult<Box<dyn ClassInstance>> {
        let class = self.class().unwrap();
        let storage_size = self.storage_size().unwrap();
        let mut core = self.core.clone();
        let instance = Self::instantiate(&mut core, &class, storage_size).unwrap();
        let mut fields = vec![0; storage_size];
        if storage_size != 0 {
            core.read_bytes(self.ptr_fields().unwrap(), &mut fields).unwrap();
            core.write_bytes(instance.ptr_fields().unwrap(), &fields).unwrap();
        }
        let extension_size = class.extension_word_count().unwrap() * size_of::<LgtJvmWord>();
        if extension_size != 0 {
            let mut extension = vec![0; extension_size];
            core.read_bytes(self.extension_address(&class).unwrap(), &mut extension).unwrap();
            core.write_bytes(instance.extension_address(&class).unwrap(), &extension).unwrap();
        }
        Ok(Box::new(instance))
    }

    fn class_definition(&self) -> Box<dyn ClassDefinition> {
        Box::new(self.class().unwrap())
    }

    fn equals(&self, other: &dyn ClassInstance) -> JvmResult<bool> {
        Ok(other
            .as_any()
            .downcast_ref::<JavaClassInstance>()
            .is_some_and(|other| self.ptr_raw == other.ptr_raw))
    }

    fn get_field(&self, field: &dyn Field) -> JvmResult<JavaValue> {
        debug_assert!(!field.access_flags().contains(FieldAccessFlags::STATIC));
        let field_type = JavaType::parse(&field.descriptor());
        let (word_index, address) = if let Some(field) = field.as_any().downcast_ref::<JavaField>() {
            (field.word_index().unwrap(), self.java_field_address(field).unwrap())
        } else {
            let word_index = field.as_any().downcast_ref::<JavaReferenceField>().unwrap().word_index;
            (word_index, self.field_address(word_index).unwrap())
        };
        let low = read_generic(&self.core, address).unwrap();
        let codec = JavaValueCodec::new(&self.core);

        // Guest code owns these words and the reference layout is inferred (field tables or the
        // compiler's reference bitmap), so a word the collector is told is a reference can hold
        // something else. Decoding it would panic in the middle of a collection; report it and
        // treat it as null instead.
        if matches!(field_type, JavaType::Class(_) | JavaType::Array(_)) && low != 0 && !Self::is_live_instance(&self.core, low) {
            let holder = self.class().map(|class| ClassDefinition::name(&class).into_owned()).unwrap_or_default();
            tracing::error!(
                "{holder} {:#x} field {}{} (word {word_index}) holds {low:#x}, which is not a live object; reading it as null",
                self.ptr_raw,
                field.name(),
                field.descriptor()
            );
            return Ok(JavaValue::Object(None));
        }

        Ok(if matches!(field_type, JavaType::Long | JavaType::Double) {
            let high = read_generic(&self.core, address + 4).unwrap();
            codec.decode_wide(low, high, &field_type)
        } else {
            codec.decode_word(low, &field_type)
        })
    }

    fn put_field(&mut self, field: &dyn Field, value: JavaValue) -> JvmResult<()> {
        debug_assert!(!field.access_flags().contains(FieldAccessFlags::STATIC));
        let address = if let Some(field) = field.as_any().downcast_ref::<JavaField>() {
            self.java_field_address(field).unwrap()
        } else {
            self.field_address(field.as_any().downcast_ref::<JavaReferenceField>().unwrap().word_index)
                .unwrap()
        };
        let codec = JavaValueCodec::new(&self.core);

        if matches!(value, JavaValue::Long(_) | JavaValue::Double(_)) {
            let (low, high) = codec.encode_wide(&value);
            write_generic(&mut self.core, address, low).unwrap();
            write_generic(&mut self.core, address + 4, high).unwrap();
        } else {
            write_generic(&mut self.core, address, codec.encode_word(&value)).unwrap();
        }
        Ok(())
    }
}

impl Debug for JavaClassInstance {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:#x}", self.ptr_raw)
    }
}

impl Hash for JavaClassInstance {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.ptr_raw.hash(state);
    }
}
