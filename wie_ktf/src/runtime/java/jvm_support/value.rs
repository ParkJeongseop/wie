use alloc::boxed::Box;

use jvm::{JavaType, JavaValue};

use wie_core_arm::ArmCore;

use super::{KtfJvmWord, array_class_instance::JavaArrayClassInstance, class_instance::JavaClassInstance};

pub trait JavaValueExt {
    fn from_raw(raw: KtfJvmWord, r#type: &JavaType, core: &ArmCore) -> JavaValue;
    fn from_raw64(raw: KtfJvmWord, raw_high: KtfJvmWord, r#type: &JavaType) -> JavaValue;
    fn as_raw(&self) -> KtfJvmWord;
    fn as_raw64(&self) -> (KtfJvmWord, KtfJvmWord);
}

impl JavaValueExt for JavaValue {
    fn from_raw(raw: KtfJvmWord, r#type: &JavaType, core: &ArmCore) -> JavaValue {
        match r#type {
            JavaType::Void => JavaValue::Void,
            JavaType::Boolean => JavaValue::Boolean(raw != 0),
            JavaType::Byte => JavaValue::Byte(raw as i8),
            JavaType::Short => JavaValue::Short(raw as i16),
            JavaType::Int => JavaValue::Int(raw as i32),
            JavaType::Float => JavaValue::Float(f32::from_bits(raw)),
            JavaType::Char => JavaValue::Char(raw as u16),
            JavaType::Class(_) => {
                if raw != 0 {
                    let instance = JavaClassInstance::from_raw(raw, core);
                    // Apps sometimes pass garbage as an object argument; treat
                    // an unreadable instance as null instead of aborting.
                    match instance.class().and_then(|x| x.name()) {
                        Ok(name) => {
                            if name.starts_with('[') {
                                let instance = JavaArrayClassInstance::from_raw(raw, core);
                                JavaValue::Object(Some(Box::new(instance)))
                            } else {
                                JavaValue::Object(Some(Box::new(instance)))
                            }
                        }
                        Err(e) => {
                            tracing::warn!("invalid object pointer {raw:#x} passed as java value: {e:?}");
                            JavaValue::Object(None)
                        }
                    }
                } else {
                    JavaValue::Object(None)
                }
            }
            JavaType::Array(_) => {
                if raw != 0 {
                    // Apps sometimes pass a non-array object where an array is
                    // declared (e.g. a String to String.<init>([CII)V in a
                    // rarely-taken error path). Wrapping it as an array made
                    // downstream element loads fabricate values and panic; wrap it
                    // as a plain instance instead so the JVM throws a catchable
                    // IllegalArgumentException ("Not an array") like real bytecode
                    // misuse would surface on hardware.
                    let instance = JavaClassInstance::from_raw(raw, core);
                    match instance.class().and_then(|x| x.name()) {
                        Ok(name) if !name.starts_with('[') => {
                            let (pc, lr) = core.read_pc_lr().unwrap_or((0, 0));
                            tracing::warn!("non-array object {raw:#x} of class {name} passed as array argument (pc={pc:#x}, lr={lr:#x})");
                            JavaValue::Object(Some(Box::new(instance)))
                        }
                        Ok(_) => JavaValue::Object(Some(Box::new(JavaArrayClassInstance::from_raw(raw, core)))),
                        Err(e) => {
                            tracing::warn!("invalid object pointer {raw:#x} passed as array java value: {e:?}");
                            JavaValue::Object(None)
                        }
                    }
                } else {
                    JavaValue::Object(None)
                }
            }
            _ => panic!(),
        }
    }

    fn from_raw64(raw: KtfJvmWord, raw_high: KtfJvmWord, r#type: &JavaType) -> JavaValue {
        match r#type {
            JavaType::Long => JavaValue::Long((((raw_high as u64) << 32) | raw as u64) as i64),
            JavaType::Double => JavaValue::Double(f64::from_bits(((raw_high as u64) << 32) | raw as u64)),
            _ => panic!(),
        }
    }

    fn as_raw(&self) -> KtfJvmWord {
        match self {
            JavaValue::Void => 0,
            JavaValue::Boolean(x) => *x as _,
            JavaValue::Byte(x) => *x as _,
            JavaValue::Short(x) => *x as _,
            JavaValue::Int(x) => *x as _,
            JavaValue::Float(x) => x.to_bits() as _,
            JavaValue::Char(x) => *x as _,
            JavaValue::Object(x) => {
                if let Some(x) = x {
                    if let Some(x) = x.as_any().downcast_ref::<JavaClassInstance>() {
                        x.ptr_raw as _
                    } else if let Some(x) = x.as_any().downcast_ref::<JavaArrayClassInstance>() {
                        x.class_instance.ptr_raw as _
                    } else {
                        unreachable!()
                    }
                } else {
                    0
                }
            }
            _ => panic!(),
        }
    }

    // (low, high)
    fn as_raw64(&self) -> (KtfJvmWord, KtfJvmWord) {
        match *self {
            JavaValue::Long(x) => ((x as u64 & 0xFFFFFFFF) as _, (x as u64 >> 32) as _),
            JavaValue::Double(x) => ((x.to_bits() & 0xFFFFFFFF) as _, (x.to_bits() >> 32) as _),
            _ => panic!(),
        }
    }
}
