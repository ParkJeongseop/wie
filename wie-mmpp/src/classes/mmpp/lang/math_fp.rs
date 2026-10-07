use alloc::{format, string::String as RustString, vec};

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult, runtime::JavaLangString};
use jvm_class_proto::JavaMethodProto;
use jvm_types::{ClassAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::lang::String;

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class mmpp.lang.MathFP
//
// 20.12 fixed point in an int: the conversions and arithmetic. The transcendental functions of
// the API (pow, sqrt, log, exp and the trigonometry) are not implemented.
pub struct MathFP;

impl MathFP {
    const FRACTION_BITS: u32 = 12;
    const ONE: i64 = 1 << Self::FRACTION_BITS;
    const MAX_VALUE_INT: i64 = (1 << 19) - 1;
    const MIN_VALUE_INT: i64 = -(1 << 19);

    pub fn as_proto() -> WieJavaClassProto {
        let flags = MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC;

        WieJavaClassProto {
            name: "mmpp/lang/MathFP",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("abs", "(I)I", Self::abs, flags),
                JavaMethodProto::new("round", "(I)I", Self::round, flags),
                JavaMethodProto::new("max", "(II)I", Self::max, flags),
                JavaMethodProto::new("min", "(II)I", Self::min, flags),
                JavaMethodProto::new("add", "(II)I", Self::add, flags),
                JavaMethodProto::new("sub", "(II)I", Self::sub, flags),
                JavaMethodProto::new("multiply", "(II)I", Self::multiply, flags),
                JavaMethodProto::new("divide", "(II)I", Self::divide, flags),
                JavaMethodProto::new("parseFP", "(I)I", Self::parse_fp, flags),
                JavaMethodProto::new("parseFP", "(Ljava/lang/String;)I", Self::parse_fp_string, flags),
                JavaMethodProto::new("toInt", "(I)I", Self::to_int, flags),
                JavaMethodProto::new("toString", "(I)Ljava/lang/String;", Self::to_string, flags),
            ],
            fields: vec![],
            access_flags: ClassAccessFlags::PUBLIC | ClassAccessFlags::FINAL,
        }
    }

    /// Checks a result against the representable range, as the API's ArithmeticException cases ask.
    async fn checked(jvm: &Jvm, value: i64) -> JvmResult<i32> {
        if value > i32::MAX as i64 || value < i32::MIN as i64 {
            return Err(jvm.exception("java/lang/ArithmeticException", "fixed point overflow").await);
        }

        Ok(value as i32)
    }

    async fn abs(_: &Jvm, _: &mut WieJvmContext, value: i32) -> JvmResult<i32> {
        Ok(value.wrapping_abs())
    }

    async fn round(_: &Jvm, _: &mut WieJvmContext, value: i32) -> JvmResult<i32> {
        Ok((((value as i64 + Self::ONE / 2) >> Self::FRACTION_BITS) << Self::FRACTION_BITS) as i32)
    }

    async fn max(_: &Jvm, _: &mut WieJvmContext, a: i32, b: i32) -> JvmResult<i32> {
        Ok(a.max(b))
    }

    async fn min(_: &Jvm, _: &mut WieJvmContext, a: i32, b: i32) -> JvmResult<i32> {
        Ok(a.min(b))
    }

    async fn add(_: &Jvm, _: &mut WieJvmContext, a: i32, b: i32) -> JvmResult<i32> {
        Ok(a.wrapping_add(b))
    }

    async fn sub(_: &Jvm, _: &mut WieJvmContext, a: i32, b: i32) -> JvmResult<i32> {
        Ok(a.wrapping_sub(b))
    }

    async fn multiply(jvm: &Jvm, _: &mut WieJvmContext, a: i32, b: i32) -> JvmResult<i32> {
        Self::checked(jvm, (a as i64 * b as i64) >> Self::FRACTION_BITS).await
    }

    async fn divide(jvm: &Jvm, _: &mut WieJvmContext, a: i32, b: i32) -> JvmResult<i32> {
        if b == 0 {
            return Err(jvm.exception("java/lang/ArithmeticException", "/ by zero").await);
        }

        Self::checked(jvm, ((a as i64) << Self::FRACTION_BITS) / b as i64).await
    }

    async fn parse_fp(jvm: &Jvm, _: &mut WieJvmContext, value: i32) -> JvmResult<i32> {
        if !(Self::MIN_VALUE_INT..=Self::MAX_VALUE_INT).contains(&(value as i64)) {
            return Err(jvm.exception("java/lang/NumberFormatException", "out of fixed point range").await);
        }

        Ok(value << Self::FRACTION_BITS)
    }

    async fn parse_fp_string(jvm: &Jvm, _: &mut WieJvmContext, value: ClassInstanceRef<String>) -> JvmResult<i32> {
        let text = JavaLangString::to_rust_string(jvm, &value).await?;
        let text = text.trim();
        let (negative, digits) = match text.strip_prefix('-') {
            Some(digits) => (true, digits),
            None => (false, text.strip_prefix('+').unwrap_or(text)),
        };
        let (integer, fraction) = digits.split_once('.').unwrap_or((digits, ""));

        let valid = !digits.is_empty() && integer.bytes().chain(fraction.bytes()).all(|digit| digit.is_ascii_digit());
        let integer_value = if integer.is_empty() { Some(0) } else { integer.parse::<i64>().ok() };
        let (true, Some(integer_value)) = (valid, integer_value) else {
            return Err(jvm.exception("java/lang/NumberFormatException", text).await);
        };
        if integer_value > Self::MAX_VALUE_INT + 1 {
            return Err(jvm.exception("java/lang/NumberFormatException", text).await);
        }

        // the fraction digit by digit, so no decimal precision is lost before scaling
        let mut numerator = 0i64;
        let mut denominator = 1i64;
        for digit in fraction.bytes().take(9) {
            numerator = numerator * 10 + (digit - b'0') as i64;
            denominator *= 10;
        }
        let magnitude = integer_value * Self::ONE + (numerator * Self::ONE + denominator / 2) / denominator;
        let fixed = if negative { -magnitude } else { magnitude };
        if !(Self::MIN_VALUE_INT * Self::ONE..=Self::MAX_VALUE_INT * Self::ONE + (Self::ONE - 1)).contains(&fixed) {
            return Err(jvm.exception("java/lang/NumberFormatException", text).await);
        }

        Ok(fixed as i32)
    }

    async fn to_int(_: &Jvm, _: &mut WieJvmContext, value: i32) -> JvmResult<i32> {
        Ok(((value as i64 + Self::ONE / 2) >> Self::FRACTION_BITS) as i32)
    }

    async fn to_string(jvm: &Jvm, _: &mut WieJvmContext, value: i32) -> JvmResult<ClassInstanceRef<String>> {
        let magnitude = (value as i64).abs();
        let fraction = (magnitude & (Self::ONE - 1)) * 10_000 / Self::ONE;
        let sign = if value < 0 { "-" } else { "" };
        let text: RustString = format!("{sign}{}.{fraction:04}", magnitude >> Self::FRACTION_BITS);

        Ok(JavaLangString::from_rust_string(jvm, &text).await?.into())
    }
}
