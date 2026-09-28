use wie_util::{Result, write_generic};

use alloc::vec::Vec;

use wipi_types::wipic::WIPICWord;

use crate::context::WIPICContext;

pub async fn handle_input(
    context: &mut dyn WIPICContext,
    key: WIPICWord,
    event_type: i32,
    ptr_committed: WIPICWord,
    ptr_committed_size: WIPICWord,
    ptr_composing: WIPICWord,
    ptr_composing_size: WIPICWord,
) -> Result<i32> {
    tracing::warn!(
        "stub MC_imHandleInput({key:#x}, {event_type}, {ptr_committed:#x}, {ptr_committed_size:#x}, {ptr_composing:#x}, {ptr_composing_size:#x})"
    );

    write_generic(context, ptr_committed_size, 0i32)?;
    write_generic(context, ptr_composing_size, 0i32)?;

    Ok(0)
}

// Input modes the platform reports, in the specification's vocabulary: an ISO 639
// language code, "/S" or "/L" for a script with case, and "N123" for digits.
// LGT titles read the first code to decide whether upper or lower case comes first
// and then select a mode by index, so the order is part of the contract.
const INPUT_MODES: [&str; 4] = ["EN/L", "EN/S", "KO", "N123"];

pub async fn get_supported_mode_count(_context: &mut dyn WIPICContext) -> Result<i32> {
    tracing::debug!("MC_imGetSupportedModeCount()");

    Ok(INPUT_MODES.len() as i32)
}

/// Returns an `M_Char **` table of the mode codes. The caller keeps the table, so it
/// is allocated in guest memory once per call (games ask once, at start-up).
pub async fn get_supported_modes(context: &mut dyn WIPICContext) -> Result<WIPICWord> {
    tracing::debug!("MC_imGetSupportedModes()");

    let strings: Vec<&[u8]> = INPUT_MODES.iter().map(|x| x.as_bytes()).collect();
    let table_size = (INPUT_MODES.len() * 4) as u32;
    let total = table_size + strings.iter().map(|x| x.len() as u32 + 1).sum::<u32>();
    let table = context.alloc_raw(total)?;

    let mut cursor = table + table_size;
    for (index, string) in strings.iter().enumerate() {
        write_generic(context, table + index as u32 * 4, cursor)?;
        context.write_bytes(cursor, string)?;
        context.write_bytes(cursor + string.len() as u32, &[0])?;
        cursor += string.len() as u32 + 1;
    }

    Ok(table)
}

/// 1 when the mode was applied, 0 otherwise (the one call in this block that does not
/// answer with an error code). Input itself is not composed yet, so nothing is kept.
pub async fn set_current_mode(_context: &mut dyn WIPICContext, mode: i32) -> Result<i32> {
    tracing::debug!("MC_imSetCurrentMode({mode})");

    Ok(((0..INPUT_MODES.len() as i32).contains(&mode)) as i32)
}

pub async fn get_current_mode(_context: &mut dyn WIPICContext) -> Result<i32> {
    tracing::debug!("MC_imGetCurrentMode()");

    Ok(0)
}
