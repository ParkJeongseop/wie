use alloc::{
    borrow::ToOwned,
    boxed::Box,
    collections::{BTreeSet, btree_map::BTreeMap},
    format, str,
    string::{String, ToString},
    vec::Vec,
};

use jvm::{
    ClassInstance, Result as JvmResult,
    runtime::{JavaIoInputStream, JavaLangClassLoader, JavaLangString},
};

use wie_backend::{DefaultTaskRunner, Emulator, Event, Platform, System, extract_zip};
use wie_jvm_support::{JvmSupport, RustJavaJvmImplementation};
use wie_midp::classes::net::wie::MIDPKeyCode;
use wie_util::{Result, WieError};

pub struct J2MEEmulator {
    system: System,
}

impl J2MEEmulator {
    pub fn jar_metadata(jar: &[u8]) -> Result<Option<(String, Option<Vec<u8>>)>> {
        let files = extract_zip(jar)?;
        let Some(manifest) = files.get("META-INF/MANIFEST.MF") else {
            return Ok(None);
        };
        let descriptor = J2MEDescriptor::parse(manifest);

        if descriptor.name.is_empty() || descriptor.main_class_name.is_empty() {
            return Ok(None);
        }

        let icon = if descriptor.icon.is_empty() {
            None
        } else {
            files.get(descriptor.icon.trim_start_matches('/')).cloned()
        };

        Ok(Some((descriptor.name, icon)))
    }

    /// A JAD and its JAR in one archive, the pair a MIDlet suite is distributed as.
    pub fn loadable_archive(files: &BTreeMap<String, Vec<u8>>) -> bool {
        Self::archive_entry(files, ".jad").is_some() && Self::archive_entry(files, ".jar").is_some()
    }

    fn archive_entry<'a>(files: &'a BTreeMap<String, Vec<u8>>, extension: &str) -> Option<(&'a String, &'a Vec<u8>)> {
        files.iter().find(|(filename, _)| filename.to_ascii_lowercase().ends_with(extension))
    }

    pub fn archive_title(files: &BTreeMap<String, Vec<u8>>) -> Option<String> {
        let title = J2MEDescriptor::parse(Self::archive_entry(files, ".jad")?.1).name;
        (!title.is_empty()).then_some(title)
    }

    pub fn archive_id(files: &BTreeMap<String, Vec<u8>>) -> Option<String> {
        let name = J2MEDescriptor::parse(Self::archive_entry(files, ".jad")?.1).name;
        Some(Self::suite_id(name, Self::archive_entry(files, ".jar")?.0))
    }

    pub fn archive_icon(files: &BTreeMap<String, Vec<u8>>) -> Option<Vec<u8>> {
        let descriptor = J2MEDescriptor::parse(Self::archive_entry(files, ".jad")?.1);
        let jar = extract_zip(Self::archive_entry(files, ".jar")?.1).ok()?;

        descriptor.icon_paths().find_map(|path| jar.get(path.trim_start_matches('/')).cloned())
    }

    // A descriptor may leave even the suite's name to the manifest in the JAR.
    fn suite_id(name: String, jar_filename: &str) -> String {
        if name.is_empty() { jar_filename.to_owned() } else { name }
    }

    pub fn from_archive(platform: Box<dyn Platform>, files: BTreeMap<String, Vec<u8>>) -> Result<Self> {
        let (Some((_, jad)), Some((jar_filename, jar))) = (Self::archive_entry(&files, ".jad"), Self::archive_entry(&files, ".jar")) else {
            return Err(WieError::FatalError("Missing JAD or JAR in J2ME archive".into()));
        };

        Self::from_jad_jar(platform, jad.clone(), jar_filename.clone(), jar.clone())
    }

    pub fn from_jad_jar(platform: Box<dyn Platform>, jad: Vec<u8>, jar_filename: String, jar: Vec<u8>) -> Result<Self> {
        let descriptor = J2MEDescriptor::parse(&jad);

        tracing::info!("Loading app {}, mclass {}", descriptor.name, descriptor.main_class_name);
        if let Some((width, height)) = descriptor.display_size
            && let Err(error) = platform.screen().resize(width, height)
        {
            tracing::warn!("Ignoring unsupported display size {width}x{height}: {error}");
        }

        // A descriptor may leave the MIDlet's class to the manifest in the JAR.
        let id = Self::suite_id(descriptor.name, &jar_filename);
        let main_class_name = (!descriptor.main_class_name.is_empty()).then_some(descriptor.main_class_name);

        let files = [(jar_filename.to_owned(), jar)].into_iter().collect();
        Self::load(platform, &jar_filename, &id, main_class_name, descriptor.properties, &files)
    }

    pub fn from_jar(platform: Box<dyn Platform>, jar_filename: &str, jar: Vec<u8>) -> Result<Self> {
        let files = [(jar_filename.to_owned(), jar)].into_iter().collect();

        Self::load(platform, jar_filename, jar_filename, None, BTreeMap::new(), &files)
    }

    fn load(
        platform: Box<dyn Platform>,
        jar_filename: &str,
        id: &str,
        main_class_name: Option<String>,
        properties: BTreeMap<String, String>,
        files: &BTreeMap<String, Vec<u8>>,
    ) -> Result<Self> {
        let system = System::new(platform, id, id, DefaultTaskRunner);

        for (path, data) in files {
            system.filesystem().add_virtual(path, data.clone());
        }

        let mut system_clone = system.clone();
        let jar_filename = jar_filename.to_owned();

        system.spawn(async move || Self::do_start(&mut system_clone, jar_filename, properties, main_class_name).await);

        Ok(J2MEEmulator { system })
    }

    #[tracing::instrument(name = "start", skip_all)]
    async fn do_start(
        system: &mut System,
        jar_filename: String,
        properties: BTreeMap<String, String>,
        main_class_name: Option<String>,
    ) -> Result<()> {
        // What the platform reports to a MIDlet follows LG Telecom's ez-java handsets, the J2ME
        // platform of our titles; the values are the ones LG's own emulator carries. Its key codes
        // are the widespread J2ME ones (negative for the keys without a character), and all of its
        // Graphics are MMPP GraphicsX instances. Its fonts are 9, 11 and 15 pixels high; the usual,
        // middle one has characters 5 pixels wide, 10 for Hangul, and titles count on it, wrapping
        // their text after a fixed number of characters.
        let key_map = format!(
            "{}=-1:1,{}=-2:6,{}=-3:2,{}=-4:5,{}=-5:8,{}=-6:11,{}=-7:12,{}=-8:10,{}=-10:9,{}=-11,{}=-13,{}=-14,{}=55:9,{}=57:10,{}=42:11,{}=35:12",
            MIDPKeyCode::UP as i32,
            MIDPKeyCode::DOWN as i32,
            MIDPKeyCode::LEFT as i32,
            MIDPKeyCode::RIGHT as i32,
            MIDPKeyCode::FIRE as i32,
            MIDPKeyCode::LEFT_SOFT_KEY as i32,
            MIDPKeyCode::RIGHT_SOFT_KEY as i32,
            MIDPKeyCode::CLEAR as i32,
            MIDPKeyCode::CALL as i32,
            MIDPKeyCode::HANGUP as i32,
            MIDPKeyCode::VOLUME_UP as i32,
            MIDPKeyCode::VOLUME_DOWN as i32,
            MIDPKeyCode::KEY_NUM7 as i32,
            MIDPKeyCode::KEY_NUM9 as i32,
            MIDPKeyCode::KEY_STAR as i32,
            MIDPKeyCode::KEY_POUND as i32,
        );
        let system_properties = [
            ("microedition.configuration", "CLDC-1.0"),
            ("microedition.profiles", "MIDP-1.0"),
            ("microedition.platform", "j2me"),
            ("microedition.locale", "ko"),
            ("microedition.encoding", "KSC5601"),
            ("microedition.phone.model", "CPD525/1.0"),
            ("wie.midp.graphics", "mmpp/microedition/lcdui/GraphicsX"),
            ("wie.midp.font.sizes", "6.75:9,7.5:11,10.5:15"),
            ("wie.midp.keymap", key_map.as_str()),
        ];
        let descriptor_keys = properties.keys().cloned().collect::<BTreeSet<_>>();
        let properties = properties
            .into_iter()
            .map(|(k, v)| (format!("wie.appProperty.{k}"), v))
            .collect::<Vec<_>>();
        let properties = system_properties
            .into_iter()
            .chain(properties.iter().map(|(k, v)| (k.as_ref(), v.as_ref())))
            .collect::<Vec<_>>();

        let protos = [wie_midp::get_protos().into(), wie_mmpp::get_protos().into()];
        let jvm = JvmSupport::new_jvm(system, Some(&jar_filename), Box::new(protos), &properties, RustJavaJvmImplementation).await?;

        // The manifest in the JAR carries the suite's attributes and names its MIDlet. Attributes the
        // descriptor also gave keep the descriptor's value, except the MIDlet class: a descriptor
        // rewritten by the distributing server can name a class the JAR does not have (미니미니트레인's
        // names the title), so the descriptor's class is only the fallback.
        let class_loader = JavaLangClassLoader::get_system_class_loader(&jvm).await.unwrap();
        let manifest = match JavaLangClassLoader::get_resource_as_stream(&jvm, &class_loader, "META-INF/MANIFEST.MF")
            .await
            .unwrap()
        {
            Some(stream) => Some(J2MEDescriptor::parse(&JavaIoInputStream::read_until_end(&jvm, &stream).await.unwrap())),
            None => None,
        };

        let mut manifest_main_class_name = String::new();
        if let Some(manifest) = manifest {
            for (k, v) in manifest.properties {
                if descriptor_keys.contains(&k) {
                    continue;
                }
                let property_key = format!("wie.appProperty.{k}");
                let property_key = JavaLangString::from_rust_string(&jvm, &property_key).await.unwrap();
                let property_value = JavaLangString::from_rust_string(&jvm, &v).await.unwrap();

                let _: Option<Box<dyn ClassInstance>> = jvm
                    .invoke_static(
                        "java/lang/System",
                        "setProperty",
                        "(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/Object;",
                        (property_key, property_value),
                    )
                    .await
                    .unwrap();
            }
            manifest_main_class_name = manifest.main_class_name;
        }

        let main_class_name = if !manifest_main_class_name.is_empty() {
            manifest_main_class_name
        } else if let Some(main_class_name) = main_class_name {
            main_class_name
        } else {
            return Err(WieError::FatalError("Main class not found".into()));
        };
        let main_class_name = main_class_name.replace('.', "/");

        // Resolve with the system loader before entering the rustjar-loaded Launcher.
        if let Err(error) = jvm.resolve_class(&main_class_name).await {
            return Err(JvmSupport::to_wie_err(&jvm, error).await);
        }
        let main_class_java = JavaLangString::from_rust_string(&jvm, &main_class_name).await.unwrap();

        let result: JvmResult<()> = jvm
            .invoke_static("net/wie/Launcher", "start", "(Ljava/lang/String;)V", (main_class_java,))
            .await;

        if let Err(x) = result {
            return Err(JvmSupport::to_wie_err(&jvm, x).await);
        }

        Ok(())
    }
}

impl Emulator for J2MEEmulator {
    fn handle_event(&mut self, event: Event) {
        self.system.event_queue().push(event)
    }

    fn tick(&mut self) -> Result<()> {
        self.system.tick()
    }
}

struct J2MEDescriptor {
    name: String,
    main_class_name: String,
    icon: String,
    /// The LCD the suite was built for, when the descriptor tells.
    display_size: Option<(u32, u32)>,
    properties: BTreeMap<String, String>,
}

impl J2MEDescriptor {
    /// The suite's icons, largest first: LG's descriptors list bigger ones beside the MIDlet's own.
    fn icon_paths(&self) -> impl Iterator<Item = &String> {
        ["MIDletX-Big-Icon", "MIDletX-Medium-Icon"]
            .into_iter()
            .filter_map(|key| self.properties.get(key))
            .chain([&self.icon])
    }

    pub fn parse(data: &[u8]) -> Self {
        let lines = data.split(|x| *x == b'\n');

        let mut name = String::new();
        let mut main_class_name = String::new();
        let mut icon = String::new();
        let mut midlet_icon = String::new();
        let mut properties = BTreeMap::new();
        let mut logical_lines: Vec<String> = Vec::new();

        for line in lines {
            let Ok(line) = str::from_utf8(line) else {
                continue;
            };
            let line = line.trim_end_matches('\r');

            if let Some(continuation) = line.strip_prefix(' ') {
                if let Some(previous) = logical_lines.last_mut() {
                    previous.push_str(continuation);
                }
            } else {
                logical_lines.push(line.to_string());
            }
        }

        for line in logical_lines {
            let line = line.trim();

            if line.is_empty() {
                continue;
            }

            let Some((key, value)) = line.split_once(':') else {
                continue;
            };
            let key = key.trim();
            let value = value.trim();
            properties.insert(key.to_string(), value.to_string());

            match key {
                "MIDlet-Name" => name = value.to_string(),
                "MIDlet-Icon" => icon = value.to_string(),
                "MIDlet-1" => {
                    let mut values = value.split(',');
                    values.next();
                    if let Some(value) = values.next() {
                        midlet_icon = value.trim().to_string();
                    }
                    if let Some(value) = values.next() {
                        main_class_name = value.trim().to_string();
                    }
                }
                _ => {}
            }
        }

        if icon.is_empty() {
            icon = midlet_icon;
        }

        // LG ez-java descriptors carry `MIDletX-` attributes. `MIDletX-LCD-Size: width,height` is the
        // LCD the build targets; the builds without it are those for the first, 120x143 handsets.
        let display_size = match properties.get("MIDletX-LCD-Size") {
            Some(size) => size
                .split_once(',')
                .and_then(|(width, height)| Some((width.trim().parse().ok()?, height.trim().parse().ok()?))),
            None if properties.keys().any(|key| key.starts_with("MIDletX-")) => Some((120, 143)),
            None => None,
        };

        Self {
            name,
            main_class_name,
            icon,
            display_size,
            properties,
        }
    }
}

#[cfg(test)]
mod tests {
    use alloc::{vec, vec::Vec};

    use super::{J2MEDescriptor, J2MEEmulator};

    #[test]
    fn parses_utf8_midlet_metadata() {
        let descriptor = J2MEDescriptor::parse(
            "Manifest-Version: 1.0\r\nMIDlet-Name: 모바일 앱\r\nMIDlet-Icon: /app.png\r\nMIDlet-1: 모바일 앱, /midlet.png, example.Main\r\n"
                .as_bytes(),
        );

        assert_eq!(descriptor.name, "모바일 앱");
        assert_eq!(descriptor.main_class_name, "example.Main");
        assert_eq!(descriptor.icon, "/app.png");
    }

    #[test]
    fn reads_the_lcd_size_of_ez_java_descriptors() {
        let sized = J2MEDescriptor::parse(b"MIDlet-Name: A\nMIDletX-No-Command: true\nMIDletX-LCD-Size: 176,200\n");
        assert_eq!(sized.display_size, Some((176, 200)));

        let first_generation = J2MEDescriptor::parse(b"MIDlet-Name: A\nMIDletX-No-Command: true\n");
        assert_eq!(first_generation.display_size, Some((120, 143)));

        let plain = J2MEDescriptor::parse(b"MIDlet-Name: A\nMIDlet-1: A, , a.Main\n");
        assert_eq!(plain.display_size, None);
    }

    #[test]
    fn reads_title_id_and_icons_of_an_archive() {
        let named = "MIDlet-Name: 게임\nMIDlet-1: 게임,/sicon.png,Main\nMIDletX-Big-Icon: /bicon.png\n";
        let files = [("DESC.jad".into(), named.as_bytes().to_vec()), ("1.jar".into(), vec![])].into();
        assert_eq!(J2MEEmulator::archive_title(&files).as_deref(), Some("게임"));
        assert_eq!(J2MEEmulator::archive_id(&files).as_deref(), Some("게임"));
        assert_eq!(
            J2MEDescriptor::parse(named.as_bytes()).icon_paths().collect::<Vec<_>>(),
            ["/bicon.png", "/sicon.png"]
        );

        let files = [("DESC.jad".into(), b"MIDlet-1: ,/sicon.png,Main\n".to_vec()), ("1.jar".into(), vec![])].into();
        assert_eq!(J2MEEmulator::archive_title(&files), None);
        assert_eq!(J2MEEmulator::archive_id(&files).as_deref(), Some("1.jar"));
    }

    #[test]
    fn ignores_malformed_manifest_lines() {
        let descriptor = J2MEDescriptor::parse(b"invalid line\nMIDlet-1: incomplete\n\xff\nMIDlet-Name: Valid App\n");

        assert_eq!(descriptor.name, "Valid App");
        assert!(descriptor.main_class_name.is_empty());
    }

    #[test]
    fn parses_midlet_icon_and_continued_manifest_value() {
        let descriptor =
            J2MEDescriptor::parse(b"MIDlet-Name: Boulder Dash\nMIDlet-1: Boulder Dash, icon.png, net.instantcom.boulderdash.BoulderDa\n shMIDlet\n");

        assert_eq!(descriptor.main_class_name, "net.instantcom.boulderdash.BoulderDashMIDlet");
        assert_eq!(descriptor.icon, "icon.png");
    }
}
