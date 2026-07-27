# API Coverage

This document tracks how completely the emulator implements the API surfaces
visible to emulated apps. It is an audit of the API crates (`wie_wipi_c`,
`wie_wipi_java`, `wie_skvm`, `wie_midp`) measured against the reference
specifications, and a prioritized list of the biggest gaps to fill in.

Status tags used below:
- **IMPL** — real logic
- **STUB** — empty, returns a dummy/default, or only logs (`tracing::warn`) without doing the work
- **MISSING** — the class/function is not defined at all

## Reference Sources

See also `CONTRIBUTING.md`.

| Surface | Reference | Notes |
|---------|-----------|-------|
| WIPI Java (`org.kwis.*`) | [WIPI Java API 1.1.1](https://nikita36078.github.io/J2ME_Docs/docs/WIPI_API_1_1_1) | Javadoc, 2003 |
| KTF extensions | [KTF WIPI API](https://nikita36078.github.io/J2ME_Docs/docs/KTF_WIPI_API) | `com.ktf.kfc`, `wec` |
| LGT extensions | [LG MMPP API](https://nikita36078.github.io/J2ME_Docs/docs/LG_MMPP_API) | `mmpp.*` |
| SKVM / SKT (`com.skt.m.*`, `com.xce.*`) | SKVM API (web.archive, xce.co.kr) | 24 classes |
| WIPI-C ABI | WIPI 1.2.1 Spec + `wipi_types` / `wipic_sys` SDK | Spec server currently down; SDK is the working reference |
| MIDP / CLDC | [MIDP 2.0](https://nikita36078.github.io/J2ME_Docs/docs/midp-2.0), CLDC 1.1 | `wie_wipi_java` and `wie_midp` build on this |

## Coverage Summary

| Crate | Surface | Implemented | Biggest gaps |
|-------|---------|-------------|--------------|
| `wie_wipi_c` | WIPI-C (104 fns) | ~66% | fonts, audio control, network, UIC widgets |
| `wie_wipi_java` | `org.kwis.*` | ~62% | LWC widgets (20 classes missing), `msf.core`/`msf.io` networking, Display capability queries |
| `wie_skvm` | `com.skt.m.*`, `com.xce.*` | ~30% | audio volume, Device control, 3D, SMS, 10 classes missing |
| `wie_midp` | `javax.microedition.*` | ~60% | Alert/Form/Command, media.control |
| carrier ext | KTF `kfc`/`wec`, LGT `mmpp.*` | 0% | entire namespaces unimplemented |

## WIPI-C (`wie_wipi_c`)

Grouped by `api/` submodule. Database and the core graphics pipeline are solid;
fonts, audio control, networking, and the UIC widget system are the weak spots.

- **kernel** — memory (alloc/calloc/free), timers (def/set_timer), resources,
  printk/sprintk are IMPL. STUB: `unset_timer` (cannot cancel), `get_total_memory`
  / `get_free_memory` (hardcoded), `get_cur_program_id`, `set_system_property`.
- **graphics** — framebuffer, put_pixel, fill/draw rect/line/arc, image
  decode/draw, copy_area, offscreen buffers, RGB<->pixel are IMPL. STUB: all
  font metrics (`get_font`, `get_font_height/ascent/descent` return constants).
- **database** — fully IMPL (stream + record modes, KTF variants, write-through).
- **media** — `clip_put_data`, `play`, `stop`, `vibrator` are IMPL. STUB:
  `pause`, `resume`, `record`, all volume/mute (`clip_set_volume`,
  `clip_get_volume`, `get_volume`, `set/get_mute_state`), `clip_get_type/info`.
- **net** — `connect`, `close`, `socket_close` are STUB (fail immediately).
- **uic** — entire WIPI native widget surface is STUB
  (`create_application_context`, `get_class`, `create`, `destroy`, `get_menu_item`).
- **util** — `htons` IMPL. **misc** — `back_light` STUB.

## WIPI Java (`wie_wipi_java`, `org.kwis.*`)

Two kinds of gap: stubbed methods on existing classes, and classes that are not
defined at all (measured against WIPI Java API 1.1.1).

### Stubbed methods on existing classes

- **lcdui.Graphics** — mostly IMPL; STUB: `getPixel`, `get/setPixels`,
  `getRGBPixels`, `fillPolygon`, `drawPolygon`, `encodeImage` (partial).
- **lcdui.Image** — core IMPL; STUB: `loadImage`, `createSubImage`, animation
  (`isAnimated`/`play`/`stop`), `setTransparentColor`.
- **lcdui.Display** — capability queries now report against the 16bpp RGB565
  color framebuffer: `isColor` (true), `numColors` (65536), `getBitsPerPixel`
  (16), `hasRepeatEvents` (true — the runtime delivers key repeat). Still STUB:
  `hasPointerEvents`/`hasPointerMotionEvents` (false — no pointer events in the
  engine yet), `getKeyName`, `flush`, `grabKey`/`ungrabKey`, listeners.
- **media.Clip** — construction + `getType` IMPL; STUB: `setVolume`,
  `setPosition`, `getPosition`, `setStopTime`, `getStopTime`, `getVolume`,
  `setListener`, `setBuffer`.
- **media.Player** — `play(Clip)`/`stop(Clip)` IMPL; `BaseClip` variants
  (pause/stop/resume/play/record) STUB.
- **media.Volume** — `set`/`get` IMPL; mute + default-volume STUB.
- **db.DataBase** — CRUD IMPL; STUB: `sortRecord`, `getAccessMode`,
  `getDataBaseSize`, `getLastModified`, `deleteDataBase(String,int)`.
- **lwc.Component / TextComponent** — all STUB.
- **handset.BackLight** — all STUB.

### Missing classes (defined in 1.1.1, absent here)

- **org.kwis.msp.lwc** (16): ButtonComponent, ChangeListener,
  CheckboxComponent, CheckboxGroup, Command, CommandBarComponent,
  CommandListener, DateFieldComponent, Decorator, DialogComponent,
  ImageComponent, ListComponent, ListItemComponent, ProgressComponent,
  ProxyCard, ScrollbarComponent, TickerComponent. (Defined now:
  GrabKeyListener, ActionListener, FormComponent (extends
  ContainerComponent), LabelComponent (extends Component) — the last two
  unblocked KTF boot crashes.)
- **org.kwis.msp.lcdui** (3): DisplayProxy, JletStateChangeException,
  SystemEventListener. (InputMethodListener defined — interface
  `void notifyTextChanged(char[],int,int)`; unblocked KTF boot crashes.)
- **org.kwis.msp.handset** (1): Call. (LED defined: static getCount/set/get.)
- **org.kwis.msp.media** (0): MediaUnsupportedException defined (extends
  RuntimeException).
- **wec (KTF)** — OEMDevice defined (static getAddressBook/getSYSTheme,
  return null = unsupported). AddressBook/SYSTheme still missing.
- **org.kwis.msp.db** (3): DataComparatorInteger, DataComparatorString,
  DataFilterInteger.
- **org.kwis.msf.core** (3, whole package): Kernel, ProgramExitException, Shared.
- **org.kwis.msf.io** (4): HttpSocket, Socket, URL, Message.

## SKVM / SKT (`wie_skvm`)

Measured against the SKVM API archive (24 classes). File I/O and Vibration are
complete; audio and device control are weak.

- **IMPL**: XFile / FileInputStream / FileOutputStream (file I/O), Vibration,
  MathFP, Graphics2D (`drawImage`, `createMaskableImage`), Toolkit,
  `WieAudioClip` playback (open/play/loop/stop/close via backend SMAF audio),
  ByteToCharEUC_KR (undocumented xce EUC-KR decoder; games construct it for
  Korean text).
- **STUB**: AudioSystem volume, Device (setColorMode/backlight/keytone/…;
  isKeyToneEnabled exists but returns a constant), BackLight, ProgressBar,
  XTextField (input/paint), XDisplay (copyLCD/refresh), Graphics2D.captureLCD,
  XFile.fsavail (reports a constant 10MB free).
- XDisplay.drawImageEx (masked-image blit) is now IMPL — draws a source
  region skipping pixels whose mask pixel is black.
- **MISSING** (10, in the spec but not implemented): Graphics3D, Object3D
  (3D), SMS, SMSListener, SMSMessage (messaging), Call, PhoneBook, SISImage,
  ResourceAllocException, UserStopException.

## MIDP (`wie_midp`, `javax.microedition.*`)

`wie_wipi_java` is built on top of this layer, so its gaps affect WIPI too.

- **IMPL**: Graphics (drawing; honors the current Font size in draw/metrics),
  Font (stores the point size; getHeight/stringWidth derive from it), Image,
  Canvas / GameCanvas (repaint is null-safe before the canvas is shown),
  Displayable.isShown, Display lifecycle, RecordStore CRUD, SmafPlayer,
  MIDlet init.
- **STUB**: Alert (setType/setTimeout/setString), Form (append), Command
  dispatch, ChoiceGroup, `serviceRepaints`, `notifyDestroyed`,
  RecordStore delete/list, Font face/style rendering (size only).
- **MISSING / thin**: `media.control`, most of `io` (Generic Connection),
  `pki`; Manager only handles SMAF.

## Platform Runtime Gaps (empirical)

Not API-crate surface, but blockers found while running real games:

- **LGT stdlib imports** — the C library import table is partially identified.
  Known: sprintf (0x3f7, identified via argument probing), strcpy/strcat/…,
  memcpy/memset, time/localtime. Unknown: 0x415 (two nearby pointer args,
  called ~70x at startup; wrong semantics eventually corrupt the app heap —
  blocks 데몬헌터) and 0x404 (single seed-like call, srand-shaped; stubbed).
- **LGT WIPI-C SVC ids** — mostly mapped now. id 0xcf (graphics block) =
  MC_grpGetContext (implemented); KTF media slot 15 = MC_mdaSetVolume. id
  0x384 (900) is polled ~every 64ms with args (0xffff, 0xff, 0xff, ptr);
  purpose unidentified, stubbed to return 0 (Unk16) so 2008베이징올림픽
  boots past it. id 0x19c (database block) stubbed to return 0 (Unk17) so
  슈퍼액션히어로3 boots past it.
- **Phone-number DRM** — some games gate on getSystemProperty("PHONENUMBER"
  / "MIN"). wie returns an empty PHONENUMBER, which *passes* the check on
  games that compare the phone number against a value (empty makes the
  comparison void). Verified with 슈퍼액션히어로3: it requires naming the
  handset to a specific number (010-5514-5031) on real hardware, but on the
  emulator the empty phone number sails through to the game (the real boot
  blocker was the unmapped SVC 0x19c above, not the DRM). Putting a real-
  looking value here instead would make such games fail authentication.
- ~~**WIPI-C text rendering (tofu)**~~ — fixed: MC_grpDrawString decoded
  strings as UTF-8, turning EUC-KR Korean into replacement glyphs. Now
  decoded as EUC-KR like the other WIPI-C string APIs. Affects every game
  that draws Korean through the native path (2008베이징올림픽 now renders
  Korean correctly).
- **KTF loader** — "wipi init failed 0xffffffff" during init is often a
  missing class the loader tries to resolve (e.g. 멋지다김밥군 needed
  org.kwis.msp.lwc.GrabKeyListener, now added). Other apps crash in
  startApp itself (루빅스큐브: Invalid memory access; 시네마타이쿤:
  NullPointerException) — some API returns a bad value/null that the game
  then dereferences; needs per-game investigation.
- **KTF bytecode-only apps** — some KTF jars ship a Java-bytecode main
  class instead of ARM AOT code (셔터-영혼의울림, 헬싱). KTF classes live as
  ARM structures and interpreter instances cannot cross the value boundary
  (wie_ktf value.rs as_raw needs an ARM pointer), so these apps cannot run
  yet; define_class_java and the boot path now report a clean error instead
  of panicking. Full support needs interpreter/ARM instance interop.
- **DRM-encrypted dumps** — some dumps contain an OMA DRM DCF container
  instead of a real jar (inner file starts with `odcf`; 정통맞고2007).
  These cannot run without the decryption key, on any emulator. The jar
  open failure is now reported as a clean ZipException instead of a panic.
- **KTF instanceof (java_check_type)** — keep the permissive
  `unk != 0 => Ok(1)` fallback despite its "is it correct?" TODO. KTF uses
  a vtable-based custom class hierarchy that jvm.is_instance cannot resolve,
  so replacing the fallback with a "correct" instanceof regressed most KTF
  games (28->16 in the sample) — verified and reverted. 루빅스큐브's
  Invalid memory access in startApp is downstream of this and is NOT
  fixable by tightening the check.
- **Thread context class loading** — Class.forName from app threads can fail
  to see app classes ("No such class: i"); blocks some SKT games after boot.
- **Guest heap pressure** — one LGT app exhausts the guest heap after long
  runs ("Allocation failure"); leak vs. heap size not yet determined.

## Carrier Extensions (not implemented)

None of the carrier-specific Java namespaces are implemented. Platform crates
`wie_ktf` / `wie_lgt` contain only boot/ARM/JVM glue.

- **KTF** — `com.ktf.kfc` (~32 GUI widgets: GButton, GList, GMenuBar, GTextField,
  GForm, GMsgBox, …), `wec` (~25 hardware: Camera, GPS, AddressBook, SubLCD,
  WakeupTimer, …), `com.ktf.ext.am`.
- **LGT** — `mmpp.media` (BackLight, Beep, LED, MediaPlayer, Vibration),
  `mmpp.media.phrase` (ringtone), `mmpp.phone`, `mmpp.microedition.lcdui`
  (GraphicsX, TextFieldX), `mmpp.lang.MathFP`.

## Priorities

Common gaps ranked by how many games they affect:

1. ~~**Fonts (MIDP)**~~ — done: `wie_midp` Font stores the point size and
   Graphics honors it. Remaining: `wie_wipi_c` `get_font*` metrics and
   face/style rendering.
2. ~~**Audio playback (SKVM)**~~ — done: `WieAudioClip` plays through the
   backend SMAF audio. Remaining: volume/mute and pause/resume across crates.
3. ~~**Display capability queries**~~ — done for the color/depth/repeat
   queries (report against the 16bpp color framebuffer). Note: only lightly
   exercised by the current 31-game sample (getBitsPerPixel by one game), so
   this was a correctness fix more than a game-unblocker. Device capability
   stubs remain.
4. **UI components** — LWC widgets, MIDP Command/Alert/Form, SKVM XTextField.
5. **Carrier extensions** — KTF `kfc`, LGT `mmpp.media`, SKT missing classes.
   Prioritize empirically (see below).

## Verifying Priorities Empirically

Static gaps are not the same as gaps that break real games. The `library/`
collection (KTF, LGT, SKT, APK) provides real apps. Running a sample from each
carrier and collecting `tracing::warn` output from STUB paths shows which
missing/stubbed APIs are actually called — fill those first rather than
implementing surface that no game exercises.

Latest full run (313 games, headless, ~6s each with a 12s wall-clock timeout): 207 run and paint (66%), 83 report an error, 21 crash, 2 hang (kbo프로야구_2009 etc). Most crashes are missing classes the KTF loader tries to resolve (add them like the ones above) and a KTF class_instance unwrap (class_instance.rs:86, 5 games). Keep this document updated in the same change set as the implementation work.
paint, 4 fail with a reported error, 1 crashes (a pre-existing KTF loader
failure). Keep this document updated in the same change set as the
implementation work it describes.
