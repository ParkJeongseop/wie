use alloc::string::String;
use core::{net::Ipv4Addr, str};

use wie_util::{Result, read_null_terminated_string_bytes};

use wipi_types::wipic::WIPICWord;

use crate::context::WIPICContext;

pub async fn htonl(_context: &mut dyn WIPICContext, val: WIPICWord) -> Result<WIPICWord> {
    tracing::debug!("MC_utilHtonl({val:#x})");

    Ok(val.swap_bytes())
}

pub async fn htons(_context: &mut dyn WIPICContext, val: WIPICWord) -> Result<WIPICWord> {
    tracing::debug!("MC_utilHtons({val})");

    Ok((val as u16).swap_bytes() as _)
}

pub async fn ntohl(_context: &mut dyn WIPICContext, val: WIPICWord) -> Result<WIPICWord> {
    tracing::debug!("MC_utilNtohl({val})");

    Ok(val.swap_bytes())
}

pub async fn ntohs(_context: &mut dyn WIPICContext, val: WIPICWord) -> Result<WIPICWord> {
    tracing::debug!("MC_utilNtohs({val})");

    Ok((val as u16).swap_bytes() as _)
}

/// Parses a dotted-quad string into an address in network byte order; -1 (INADDR_NONE) when malformed.
pub async fn inet_addr_int(context: &mut dyn WIPICContext, ptr_str: WIPICWord) -> Result<WIPICWord> {
    let text = String::from_utf8_lossy(&read_null_terminated_string_bytes(context, ptr_str)?).into_owned();
    tracing::debug!("MC_utilInetAddrInt({text:?})");

    let octets = text
        .trim()
        .split('.')
        .map(|x| x.parse::<u8>().ok())
        .collect::<Option<alloc::vec::Vec<_>>>();
    Ok(match octets.as_deref() {
        Some([a, b, c, d]) => u32::from_le_bytes([*a, *b, *c, *d]),
        _ => WIPICWord::MAX,
    })
}

pub async fn inet_addr(context: &mut dyn WIPICContext, address: WIPICWord) -> Result<WIPICWord> {
    tracing::debug!("MC_utilInetAddr({address:#x})");

    let bytes = read_null_terminated_string_bytes(context, address)?;
    let address = str::from_utf8(&bytes).ok().and_then(|value| value.parse::<Ipv4Addr>().ok());
    Ok(address.map_or(u32::MAX, |address| u32::from_le_bytes(address.octets())))
}
