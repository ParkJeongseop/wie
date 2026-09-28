use alloc::{format, string::String};
use chrono::{DateTime, Datelike, FixedOffset, TimeZone, Timelike};
use core::cmp::min;

use wie_backend::System;
use wie_core_arm::{Allocator, ArmCore, EmulatedFunction, ResultWriter, SvcId, stdlib};
use wie_util::{
    ByteRead, ByteWrite, Result, WieError, read_generic, read_null_terminated_string_bytes, write_generic, write_null_terminated_string_bytes,
};
use wie_wipi_c::api::kernel;

use crate::runtime::{SVC_CATEGORY_STDLIB, svc_ids::StdlibSvcId};

pub fn register_stdlib_svc_handler(core: &mut ArmCore, system: &System) -> Result<()> {
    async fn handle_stdlib_svc(core: &mut ArmCore, system: &mut System, id: SvcId) -> Result<()> {
        let (_, lr) = core.read_pc_lr()?;

        match id.0 {
            x if x == StdlibSvcId::Unk2 as u32 => EmulatedFunction::call(&unk2, core, &mut ()).await?.write(core, lr),
            x if x == StdlibSvcId::Sprintf as u32 => EmulatedFunction::call(&sprintf, core, &mut ()).await?.write(core, lr),
            x if x == StdlibSvcId::Vsprintf as u32 => EmulatedFunction::call(&vsprintf, core, &mut ()).await?.write(core, lr),
            x if x == StdlibSvcId::Malloc as u32 => EmulatedFunction::call(&malloc, core, &mut ()).await?.write(core, lr),
            x if x == StdlibSvcId::Free as u32 => EmulatedFunction::call(&free, core, &mut ()).await?.write(core, lr),
            x if x == StdlibSvcId::Atoi as u32 => EmulatedFunction::call(&atoi, core, &mut ()).await?.write(core, lr),
            x if x == StdlibSvcId::Rand as u32 => EmulatedFunction::call(&rand, core, system).await?.write(core, lr),
            x if x == StdlibSvcId::Srand as u32 => EmulatedFunction::call(&srand, core, system).await?.write(core, lr),
            x if x == StdlibSvcId::Strcpy as u32 => EmulatedFunction::call(&stdlib::strcpy, core, &mut ()).await?.write(core, lr),
            x if x == StdlibSvcId::Strncpy as u32 => EmulatedFunction::call(&strncpy, core, &mut ()).await?.write(core, lr),
            x if x == StdlibSvcId::Strcat as u32 => EmulatedFunction::call(&strcat, core, &mut ()).await?.write(core, lr),
            x if x == StdlibSvcId::Strncat as u32 => EmulatedFunction::call(&strncat, core, &mut ()).await?.write(core, lr),
            x if x == StdlibSvcId::Strcmp as u32 => EmulatedFunction::call(&strcmp, core, &mut ()).await?.write(core, lr),
            x if x == StdlibSvcId::Strncmp as u32 => EmulatedFunction::call(&strncmp, core, &mut ()).await?.write(core, lr),
            x if x == StdlibSvcId::Strstr as u32 => EmulatedFunction::call(&strstr, core, &mut ()).await?.write(core, lr),
            x if x == StdlibSvcId::Strlen as u32 => EmulatedFunction::call(&stdlib::strlen, core, &mut ()).await?.write(core, lr),
            x if x == StdlibSvcId::Memcpy as u32 => EmulatedFunction::call(&stdlib::memcpy, core, &mut ()).await?.write(core, lr),
            x if x == StdlibSvcId::Memset as u32 => EmulatedFunction::call(&stdlib::memset, core, &mut ()).await?.write(core, lr),
            x if x == StdlibSvcId::Time as u32 => EmulatedFunction::call(&time, core, system).await?.write(core, lr),
            x if x == StdlibSvcId::Localtime as u32 => EmulatedFunction::call(&localtime, core, &mut ()).await?.write(core, lr),
            x if x == StdlibSvcId::Unk3 as u32 => EmulatedFunction::call(&unk3, core, &mut ()).await?.write(core, lr),
            x if x == StdlibSvcId::Sprintf as u32 => EmulatedFunction::call(&sprintf, core, &mut ()).await?.write(core, lr),
            x if x == StdlibSvcId::Memmove as u32 => EmulatedFunction::call(&memmove, core, &mut ()).await?.write(core, lr),
            _ => Err(WieError::FatalError(format!("Unknown lgt stdlib import: {:#x}", id.0))),
        }
    }

    core.register_svc_handler(SVC_CATEGORY_STDLIB, handle_stdlib_svc, system)
}

async fn srand(_core: &mut ArmCore, system: &mut System, seed: u32) -> Result<()> {
    tracing::debug!("srand({seed:#x})");

    system.set_random_state(seed);
    Ok(())
}

async fn rand(_core: &mut ArmCore, system: &mut System) -> Result<u32> {
    tracing::debug!("rand()");

    let state = system.random_state();
    let state = state.wrapping_mul(1_103_515_245).wrapping_add(12_345);
    system.set_random_state(state);

    Ok((state >> 16) & 0x7fff)
}

#[allow(clippy::too_many_arguments)]
async fn sprintf(core: &mut ArmCore, _: &mut (), ptr_dst: u32, ptr_format: u32, a0: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> Result<u32> {
    tracing::debug!("sprintf({ptr_dst:#x}, {ptr_format:#x}, {a0:#x}, {a1:#x}, {a2:#x}, {a3:#x}, {a4:#x}, {a5:#x})");

    let format = read_null_terminated_string_bytes(core, ptr_format)?;
    let result = kernel::sprintf(core, &format, &[a0, a1, a2, a3, a4, a5])?;
    write_null_terminated_string_bytes(core, ptr_dst, &result)?;

    Ok(result.len() as u32)
}

async fn vsprintf(core: &mut ArmCore, _: &mut (), ptr_dst: u32, ptr_format: u32, ptr_args: u32) -> Result<u32> {
    tracing::debug!("vsprintf({ptr_dst:#x}, {ptr_format:#x}, {ptr_args:#x})");

    let format = read_null_terminated_string_bytes(core, ptr_format)?;
    // the va_list is the caller's argument words; conversions past its end read junk like on hardware
    let args = (0..6)
        .map(|i| read_generic(core, ptr_args + i * 4))
        .collect::<Result<alloc::vec::Vec<u32>>>()?;
    let result = kernel::sprintf(core, &format, &args)?;
    write_null_terminated_string_bytes(core, ptr_dst, &result)?;

    Ok(result.len() as u32)
}

// malloc/free share MC_knlAlloc's heap and header (the block size stored in the word
// before the data), so a block from either can be released through the other.
async fn malloc(core: &mut ArmCore, _: &mut (), size: u32) -> Result<u32> {
    tracing::debug!("malloc({size:#x})");

    let address = Allocator::alloc(core, size + 4)?;
    write_generic(core, address, size)?;

    Ok(address + 4)
}

async fn free(core: &mut ArmCore, _: &mut (), ptr: u32) -> Result<()> {
    tracing::debug!("free({ptr:#x})");

    if ptr == 0 {
        return Ok(());
    }
    let size: u32 = read_generic(core, ptr - 4)?;
    Allocator::free(core, ptr - 4, size + 4)
}

async fn strncpy(core: &mut ArmCore, _: &mut (), ptr_dst: u32, ptr_src: u32, size: u32) -> Result<()> {
    tracing::debug!("strncpy({ptr_dst:#x}, {ptr_src:#x}, {size:#x})");

    let src = read_null_terminated_string_bytes(core, ptr_src)?;

    let size_to_copy = min(size, src.len() as u32);
    let bytes = &src[..size_to_copy as usize];

    core.write_bytes(ptr_dst, bytes)?;

    Ok(())
}

async fn strcat(core: &mut ArmCore, _: &mut (), ptr_dst: u32, ptr_src: u32) -> Result<()> {
    tracing::debug!("strcat({ptr_dst:#x}, {ptr_src:#x})");

    let src = read_null_terminated_string_bytes(core, ptr_src)?;
    let dst = read_null_terminated_string_bytes(core, ptr_dst)?;

    let offset = dst.len();
    write_null_terminated_string_bytes(core, ptr_dst + offset as u32, &src)?;

    Ok(())
}

async fn strncat(core: &mut ArmCore, _: &mut (), ptr_dst: u32, ptr_src: u32, size: u32) -> Result<u32> {
    tracing::debug!("strncat({ptr_dst:#x}, {ptr_src:#x}, {size:#x})");

    let src = read_null_terminated_string_bytes(core, ptr_src)?;
    let dst = read_null_terminated_string_bytes(core, ptr_dst)?;

    let size_to_copy = min(size as usize, src.len());
    write_null_terminated_string_bytes(core, ptr_dst + dst.len() as u32, &src[..size_to_copy])?;

    Ok(ptr_dst)
}

async fn strcmp(core: &mut ArmCore, _: &mut (), ptr_str1: u32, ptr_str2: u32) -> Result<u32> {
    tracing::debug!("strcmp({ptr_str1:#x}, {ptr_str2:#x})");

    let str1 = read_null_terminated_string_bytes(core, ptr_str1)?;
    let str2 = read_null_terminated_string_bytes(core, ptr_str2)?;

    Ok(str1.cmp(&str2) as u32)
}

async fn atoi(core: &mut ArmCore, _: &mut (), ptr_str: u32) -> Result<u32> {
    tracing::debug!("atoi({ptr_str:#x})");

    let string = read_null_terminated_string_bytes(core, ptr_str)?;
    let string = String::from_utf8(string).unwrap();

    Ok(string.parse().unwrap_or(0))
}

async fn time(core: &mut ArmCore, system: &mut System, ptr_time: u32) -> Result<u32> {
    let epoch_seconds = (system.platform().now().raw() / 1000) as u32;
    tracing::debug!("time({ptr_time:#x}) -> {epoch_seconds}");

    if ptr_time != 0 {
        write_generic(core, ptr_time, epoch_seconds)?;
    }

    Ok(epoch_seconds)
}

// TODO is this method better suit on wie_backend?
async fn localtime(core: &mut ArmCore, _: &mut (), ptr_time: u32) -> Result<u32> {
    tracing::debug!("localtime({ptr_time:#x})");

    // TODO we need static buffer
    let result = Allocator::alloc(core, 0x2c)?;
    let time: u32 = read_generic(core, ptr_time)?;

    // TODO kst only for now
    let kst = FixedOffset::east_opt(9 * 3600).unwrap();
    let dt: DateTime<FixedOffset> = kst.timestamp_opt(time as _, 0).unwrap();

    // TODO tm struct
    write_generic(core, result, dt.second() as u32)?;
    write_generic(core, result + 0x04, dt.minute() as u32)?;
    write_generic(core, result + 0x08, dt.hour() as u32)?;
    write_generic(core, result + 0x0c, dt.day() as u32)?;
    write_generic(core, result + 0x10, (dt.month() as u32) - 1)?; // months since January
    write_generic(core, result + 0x14, (dt.year() as u32) - 1900)?; // years since 1900
    write_generic(core, result + 0x18, dt.weekday().num_days_from_sunday() as u32)?; // days since Sunday
    write_generic(core, result + 0x1c, dt.ordinal() as u32)?; // days since January 1
    write_generic(core, result + 0x20, 0u32)?; // DST flag
    write_generic(core, result + 0x24, kst.local_minus_utc() as u32)?; // timezone offset in seconds
    write_generic(core, result + 0x28, 0u32)?; // timezone abbreviation ptr

    Ok(result)
}

async fn unk2(_core: &mut ArmCore, _: &mut (), a0: u32) -> Result<()> {
    tracing::warn!("unk2({a0:#x})");

    // error exit?

    Ok(())
}

// unknown import; observed once at startup with a seed-like first argument
// (srand-shaped). Returning 0 lets games proceed.

// unknown import; observed with two nearby pointers as arguments
// (memmove/realloc-shaped). Returning 0 lets games proceed further.
async fn memmove(core: &mut ArmCore, _: &mut (), ptr_dst: u32, ptr_src: u32, len: u32) -> Result<u32> {
    tracing::debug!("memmove({ptr_dst:#x}, {ptr_src:#x}, {len:#x})");

    // staged through a host buffer, so overlapping ranges copy correctly
    let mut bytes = alloc::vec![0u8; len as usize];
    core.read_bytes(ptr_src, &mut bytes)?;
    core.write_bytes(ptr_dst, &bytes)?;

    Ok(ptr_dst)
}

async fn unk3(core: &mut ArmCore, _: &mut (), a0: u32) -> Result<()> {
    tracing::warn!("unk3({a0:#x})");

    let _: () = core.run_function(a0, &[]).await?;

    Ok(())
}

async fn strncmp(core: &mut ArmCore, _: &mut (), ptr_str1: u32, ptr_str2: u32, len: u32) -> Result<u32> {
    tracing::debug!("strncmp({ptr_str1:#x}, {ptr_str2:#x}, {len:#x})");

    let str1 = read_null_terminated_string_bytes(core, ptr_str1)?;
    let str2 = read_null_terminated_string_bytes(core, ptr_str2)?;
    let len = len as usize;
    let a = &str1[..str1.len().min(len)];
    let b = &str2[..str2.len().min(len)];

    Ok(match a.cmp(b) {
        core::cmp::Ordering::Less => u32::MAX,
        core::cmp::Ordering::Equal => 0,
        core::cmp::Ordering::Greater => 1,
    })
}

async fn strstr(core: &mut ArmCore, _: &mut (), ptr_haystack: u32, ptr_needle: u32) -> Result<u32> {
    tracing::debug!("strstr({ptr_haystack:#x}, {ptr_needle:#x})");

    let haystack = read_null_terminated_string_bytes(core, ptr_haystack)?;
    let needle = read_null_terminated_string_bytes(core, ptr_needle)?;
    let position = if needle.is_empty() {
        Some(0)
    } else {
        haystack.windows(needle.len()).position(|window| window == needle)
    };

    Ok(position.map_or(0, |position| ptr_haystack + position as u32))
}

#[cfg(test)]
mod tests {
    use alloc::boxed::Box;

    use futures::FutureExt;

    use test_utils::TestPlatform;
    use wie_backend::{DefaultTaskRunner, System};
    use wie_core_arm::ArmCore;
    use wie_util::Result;

    use super::{rand, srand};

    #[test]
    fn random_state_is_shared_by_system_clones_and_process_local() -> Result<()> {
        let mut first = ArmCore::new(false, None)?;
        let mut second = ArmCore::new(false, None)?;
        let mut first_system = System::new(Box::new(TestPlatform::new()), "", "", DefaultTaskRunner);
        let mut second_system = System::new(Box::new(TestPlatform::new()), "", "", DefaultTaskRunner);
        let mut first_system_clone = first_system.clone();

        srand(&mut first, &mut first_system_clone, 1).now_or_never().unwrap()?;
        assert_eq!(rand(&mut first, &mut first_system).now_or_never().unwrap()?, 16_838);
        assert_eq!(first_system.random_state(), 1_103_527_590);

        assert_eq!(rand(&mut second, &mut second_system).now_or_never().unwrap()?, 16_838);
        srand(&mut second, &mut second_system, 7).now_or_never().unwrap()?;
        assert_eq!(rand(&mut first, &mut first_system).now_or_never().unwrap()?, 5_758);

        Ok(())
    }
}
