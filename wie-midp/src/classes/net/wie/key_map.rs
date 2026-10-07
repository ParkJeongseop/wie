use alloc::vec::Vec;

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult, runtime::JavaLangString};
use rustjava_runtime::classes::java::lang::String;

/// Key codes and game actions as the platform's `Canvas` reports them to the application.
///
/// The MIDP layer works with `MIDPKeyCode` values internally. A platform whose handsets report
/// other codes lists them in the `wie.midp.keymap` system property as comma-separated
/// `internal=application[:action]` entries: the internal `MIDPKeyCode` value, the code `keyPressed`
/// receives for that key, and its `getGameAction` result (0 when omitted). Keys the property leaves
/// out keep their internal code; without the property nothing is translated.
pub struct KeyMap {
    entries: Vec<(i32, i32, i32)>,
}

impl KeyMap {
    pub async fn load(jvm: &Jvm) -> JvmResult<Option<Self>> {
        let key = JavaLangString::from_rust_string(jvm, "wie.midp.keymap").await?;
        let value: ClassInstanceRef<String> = jvm
            .invoke_static("java/lang/System", "getProperty", "(Ljava/lang/String;)Ljava/lang/String;", (key,))
            .await?;
        if value.is_null() {
            return Ok(None);
        }

        let value = JavaLangString::to_rust_string(jvm, &value).await?;
        let entries = value
            .split(',')
            .filter_map(|entry| {
                let (internal, rest) = entry.split_once('=')?;
                let (application, action) = rest.split_once(':').unwrap_or((rest, "0"));

                Some((
                    internal.trim().parse().ok()?,
                    application.trim().parse().ok()?,
                    action.trim().parse().ok()?,
                ))
            })
            .collect();

        Ok(Some(Self { entries }))
    }

    /// The code the application receives for an internal key code.
    pub fn application_code(&self, internal: i32) -> i32 {
        self.entries.iter().find(|entry| entry.0 == internal).map_or(internal, |entry| entry.1)
    }

    /// The internal key code behind a code the application passes back.
    pub fn internal_code(&self, application: i32) -> i32 {
        self.entries
            .iter()
            .find(|entry| entry.1 == application)
            .map_or(application, |entry| entry.0)
    }

    /// The game action of a code the application passes back.
    pub fn game_action(&self, application: i32) -> i32 {
        self.entries.iter().find(|entry| entry.1 == application).map_or(0, |entry| entry.2)
    }
}
