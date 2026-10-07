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
  CommandListener, DateFieldComponent, Decorator,
  ImageComponent, ListComponent, ListItemComponent, ProgressComponent,
  ProxyCard, ScrollbarComponent, TickerComponent. (Defined now:
  GrabKeyListener, ActionListener, FormComponent (extends
  ContainerComponent), LabelComponent (extends Component) — the last two
  unblocked KTF boot crashes — and DialogComponent, which unblocked three
  LGT links.)
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

- **LBMP image format** (2026-09-28): the SKVM LCD bitmap decoder now covers the
  whole format. Header is six LE u32s (`LBMP`, type, width, height, size, mask).
  Type 8 = RGB332, 16 = RGB565, **2 = 4-level grayscale as two 1bpp planes** (high
  then low bit, each `size` bytes). 1bpp planes — including the **transparency
  mask** appended when `mask != 0` — pack eight rows per byte column-wise (byte
  `(y/8)*width + x`, bit `y%8`, LSB = top row); a set mask bit is transparent.
  Verified bit order/polarity on real assets (삼국지연의2 text images read
  upright only LSB-first; Chaos블레이드 tile masks match the fill color 324/324
  pixels). In the SKT library 2,024 of 2,811 LBMPs carry a mask and 47 are
  grayscale, all previously drawn opaque or rejected: 삼국지연의2 map tiles and
  "Press Any Key", 웰루시아 story text/illustrations, 더팜1 (was stuck on the
  title after the grayscale decode error) now render.

- **MC_grpGetFont sizes** (2026-09-28 sweep, first 40 s of 16 KTF + 72 LGT titles):
  games ask for size 8 (SMALL) almost exclusively — 1,539 calls across 17 LGT
  titles and 메이플스토리 도적편 (KTF) — with size 0 (MEDIUM) in 7 LGT titles
  (데몬헌터, 라그나로크 바이올렛, 라테일, 미니게임천국4, 이터니티) and size 16 once
  (라테일). Every pixel-verified layout we have (메이플 KTF dialogue box, 이스핀편
  buttons) is a size-8 request, so SMALL = the 12px cell we render. `get_font`
  still returns 0 and `get_font_height` ignores the handle, so MEDIUM/LARGE
  render at 12px too; making the handle carry the size and measuring the
  medium/large cells on the titles above is the open item.
- **SK-VM key codes and system properties** (2026-09-28, from the bytecode of the
  20 SKT titles in our library via javap): games test raw key codes in
  `keyPressed` — soft keys 129/131 (Chaos블레이드, 닥터k, 더팜1, 미니동화TING,
  삼국지연의2), call/end 190/191 (닥터k, 미니고치), volume 194/195 (six titles),
  direction 141/142/145/146, fire 148, clear 8. Our old 6/7/10/-1/13/14 values
  only appeared as unrelated constants, so `MIDPKeyCode` now uses the SK-VM
  numbering for soft/call/end/volume too. Properties: every title reads
  `m.CARRIER`/`m.MIN`/`m.MODEL`/`m.VENDER` (often with `.getBytes()`, which NPEs
  on a missing key), five read `m.SK_VM` (`equals("10")` or `parseInt`), one
  `m.COLOR`; `m.MODEL`/`m.SKT_API` are now set. Verified by a soft-key probe
  on the old vs new codes: 노리타이쿤 opens its build menu and 더팜1 advances
  its dialogue on the right soft key only with 131, and 엑스맨 backs out of the
  difficulty screen only with 129. `m.SK_VM` 10 vs 20 showed no observable
  difference in the first 40 s of any SKT title, so it stays "10". **MIDlet-Key**: 11 titles
  ship a `SecureUtil` that exits with "인증키가 존재 하지 않습니다" when
  `getAppProperty("MIDlet-Key")` is empty, but every `.msd` in the library
  already carries `MIDlet-Key`/`Key2`/`Key4` from the original download, so no
  key derivation is needed for these packages (미니고치 has neither).

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
- **MC_grpDrawString y origin differs per carrier** (2026-09-28): KTF platforms
  treat `y` as the text baseline (메이플스토리 도적편 KTF dialogue box evidence,
  2026-09), LGT platforms as the cell top. Verified by rendering all 72 LGT titles
  both ways: with the baseline shift 테일즈위버 이스핀편 draws "OK" above its button,
  바이오크로니클 puts dialog titles on the header border, 무한신맞고2009 overlaps
  story text with the picture, and the LGT build of 메이플스토리 도적편 hides its
  bottom caption behind an icon — all fit with `y` = top. The conversion now lives
  in the KTF `draw_string`; wie-lgt passes `y` through. (Matches wfeature's
  independent finding; ARAM uses top for both.)
- **LGT stdlib 0x408 = strncat** (2026-09-27, user report: 테일즈위버 막시민편
  quits the moment the ITEM tab opens). The item list formats prices with
  thousands separators via `strncat(dst, src, n)` (observed calls: n=1 of
  "1000", then "," then n=3 of "000"); the slot sits between strcat (0x407)
  and strcmp (0x409). Implemented; ITEM/SKILL/COMBO/HOTKEY/QUEST/SYSTEM tabs
  all open afterwards.
- **LGT WIPI-C SVC ids** — mostly mapped now. id 0xcf (graphics block) =
  MC_grpGetContext (implemented); KTF media slot 15 = MC_mdaSetVolume.
  **0x384..0x389 is the MC_util block in specification order** (Htonl, Htons,
  Ntohl, Ntohs, InetAddrInt, InetAddrStr — the same order as the KTF util
  table): 슈퍼액션히어로3 passes 0x385 the port 2508 and 블레이드마스터4 passes
  0x388 a pointer to a dotted-quad string right before its socket connect
  (2026-09-28). The earlier "0x384 polled every 64ms with 0xffff" stub
  (2008베이징올림픽) was therefore htonl(0xffff); it now returns the swapped
  value. 0x389 (InetAddrStr) stays unmapped until a caller shows its shape.
  id 0x19c is MC_dbListDatabases (upstream, 2026-09 sync; formerly our
  return-0 stub) — 슈퍼액션히어로3 boots past it either way.
- **MC_im block** (2026-09-28): LGT WIPI-C 300..304 is the input-method group in
  specification order — GetSupportedModeCount, GetSupportedModes, SetCurrentMode,
  GetCurrentMode, HandleInput — and the KTF graphics table carries the same five
  at 37..41. 제노니아1 (and 12 other LGT titles in the same cluster) calls
  count → modes at start-up, reads the first mode code through the returned
  `M_Char **`, tests it for "/L" or "/S" and then SetCurrentMode(2); with the
  old return-0 stubs it dereferenced NULL. The shared implementation answers
  the specification's vocabulary ("EN/L", "EN/S", "KO", "N123"); input is still
  not composed (HandleInput stays a stub).
- **LGT WIPI-C 1100 = MC_dbListDataBases(buf, len)** (2026-09-28): 뮤직팩토리 asks
  for the database name list at start-up. The repository cannot enumerate names
  yet, so the answer is the empty list (two NULs, count 0) — OpenDatabase still
  finds existing databases by name. (412 stays upstream's available-storage
  answer; the two ids are different calls.) 뮤직팩토리 then stops at unmapped
  db id 410.
- **LGT Java (Jlet) titles that fault inside their own code** (2026-09-28, with
  method-named WieErrors): 체스마스터, 레전드오브마스터, 학교가는길 read address 0
  inside the app's `startApp(String[])`, 배틀몬스터 inside a class `<init>()`.
  These are AOT-compiled Java methods invoked by wie, so something wie is
  expected to prepare (a static, a class object, a linked field) is still null —
  a disassembly task, not a missing API.
- **LGT stdlib 0x3f9 = vsprintf, 0x426 = malloc, 0x428 = free** (2026-09-28):
  당구마스터2010 calls 0x3f9 with (buffer, "han2bit.Dat", va_list) before
  loading that resource; 리얼사커매니저2009 calls 0x426 with 140/68/17 and
  keeps the pointer. malloc/free share MC_knlAlloc's heap and size header so
  blocks can cross between them. **0x415 = memmove, 0x40a = strncmp** (were
  return-0 stubs): 당구마스터2010 calls 0x415 with (dst, src, 0x17) between
  its memcpy and memset slots, matching string.h declaration order. (Slot ids
  cross-checked against wfeature's MIT table; behaviour implemented from our
  own traces.)
- **LWC members LGT titles resolve by name** (2026-09-28): `Component.hasFocus`
  (월드장기체스), `ShellComponent.serviceRepaints` (학교가는길),
  `TextBoxComponent.<init>(String,int,int)` (붕어빵타이쿤3),
  `TextComponent.iMode` field (레전드오브마스터),
  `InputMethodHandler.getCurrentMode`/`hideSymbolCard` (서든어택포켓,
  훼밀리마트타이쿤), plus `TextComponent.maxLength`/`m_td` (the text as a
  char[] — setString/getString now store and read it). Focus and input mode
  are tracked in fields; the rest are no-op stubs. Still open:
  `java/lang/Thread` vtable index 13 (메이플스토리2007 — the LGT ABI table only
  pins start=10 and setPriority=14), `org/kwis/msp/lwc/DialogComponent`
  (붕어빵타이쿤3), `wec/SYSTheme` (월드장기체스).
- **LGT Java vtable tables: Runtime/DataInputStream/Object/String** (2026-09-29):
  Java-linked LGT titles (`.raptor` lists `cldc wipijava …`) dispatch library
  methods through compiler-fixed vtable slots (`ldr r3,[obj]; ldr r12,[r3,#4*(i+1)]`),
  so a class missing from `data/lgt_java_abi.toml` gets only its parent's
  entries and slots past the end read 0 → `jump to unmapped pc 0x0`. 체스마스터,
  배틀몬스터, 학교가는길, 당신은골프왕(39 sites), 놈3, 일지매영웅전기 all call
  `Runtime.getRuntime()` then slot 13, i.e. `gc()`; 배틀몬스터 stores slot-25 of a
  `DataInputStream` into a `short[]` (`readShort`); 배틀몬스터 `Game.startApp`
  dispatches Object slot 5 (`notify`); 일지매 hits `String` slot 21 (`indexOf(I)`).
  The CLDC 1.1 declaration order — with Object overrides folded into their Object
  slots and word 0 being the compiler's instance-initializer callback — reproduces
  every previously confirmed index (Object 1/3/4, String 10/11/14/28/33/34,
  DataInputStream 23, InputStream 10-12/14/15), so the tables now carry the
  full CLDC order for Object, String, Runtime (11-13; exit=10 left unmapped on
  purpose) and DataInputStream (23-29). Thread follows it too once CLDC 1.1's
  `interrupt()` is counted (start 10, run 11, interrupt 12, isAlive 13,
  setPriority 14, getPriority 15, join 16, getName 17 — both confirmed indices
  land), which names 메이플스토리2007's slot-13 call as `isAlive()`. Result:
  체스마스터, 배틀몬스터, 일지매영웅전기 boot to their notice screens.
- **`org.kwis.msp.lwc.DialogComponent`** (2026-09-29): defined per the 1.1.1
  field list (extends ShellComponent; TYPE_NONE/OK/OK_CANCEL = 0/1/2,
  DLG_TIMEOUT/OK/CANCEL = 10/11/12, OK_BUTTON/CANCEL_BUTTON = 20/21,
  TIMEOUT_INFINITE = -1; a button-less dialog's documented display time is 3 s).
  당신은골프왕, 붕어빵타이쿤3 and 슈퍼액션히어로 all failed to link on the missing
  class; each imports only the 3-argument constructor, `doModal` and (two of them)
  `setButtonString`, and none reaches `doModal` in a 40-second boot. LWC does not
  paint, so `doModal` cannot show the dialog: it answers DLG_TIMEOUT for
  TYPE_NONE and DLG_OK otherwise, logging the title as a stub.
- **LGT Java import 0x64/0xfd = long-array store** (2026-09-29): `(long[] a, int
  i, int hi, int lo)`; 슈퍼액션히어로 widens an int with `asrs r5, r4, #31` and
  passes `r5` in r2, and stores `System.currentTimeMillis()`'s r0/r1 as r3/r2.
  Other unmapped entries of the same table seen in LGT binaries (not yet
  reached at runtime): 0x26 (7 titles), 0x38 (7), 0x40 (16), 0x5b (4 — the same
  four as 0xfd; returns a 64-bit value from `(r0, r1 & 0xff)`), 0x64 (16).
- **`java.util.Calendar.get(int)` = slot 19** (2026-09-29): 붕어빵타이쿤3 fills an
  `int[4]` from slot 19 with 1, 2, 5, 11 (YEAR, MONTH, DATE, HOUR_OF_DAY); the
  missing slot read the next heap word (string bytes "dgor") as a target. Only
  this slot is pinned. The title now runs past its notice to the Com2uS logo and
  its "속도 체크 중" screen.
- **Native-class instance fields live in an extension block** (2026-10-01): titles
  subclass wie's classes with layouts fixed by the LGT compiler against the
  handset's runtime — a Thread subclass puts its first field at word 2, Card
  subclasses at word 8, 12 or 13, Jlet at 5 or 6 (measured from field metadata
  across the library), while wie's Thread used words 0-8 and Card 0-16, so the
  two sides overwrote each other (스파이더맨3's `c extends Thread` read 1 =
  wie's `started` flag where it expected a reference). wie-defined instance
  fields now live in a per-instance extension block hung off the instance
  header's spare word (`unk1`, zero in the ABI), indexed per native class chain
  and recorded in descriptor `unk7`; only fields pinned in the ABI table stay in
  the app-visible storage, because compiled code reads those at a fixed word
  (java/lang/Class) or titles import them through the link tables (Font.face/
  style/size, Card.x/y/w/h, TextComponent.*). An imported instance field that
  is not pinned now fails the link with a message naming it.
- **GC roots from the guest** (2026-10-01): RustJava gained
  `Jvm::set_extra_roots`; the LGT runtime reports every register of every ARM
  thread context, of every guest caller suspended inside a nested
  `run_function` (their registers only exist in a Rust local while the callee
  runs — this was the reference that kept dying), and every word of every
  thread stack (whole stacks: a task suspended inside guest code keeps frames
  the live stack pointer does not cover). Words that pass the live-object
  header check become roots. 일지매영웅전기 went from 968 "not a live object"
  reports per run to 0 and 스파이더맨3/놈3 stopped panicking; 메이플스토리2007
  still reports a few dozen, so a root category is still missing there.
- **Resource streams are DataInputStreams on LGT** (2026-10-01): 슈퍼액션히어로
  dispatches slot 32 on the stream `Class.getResourceAsStream` returns and
  hands the result to `String.getBytes` — `DataInputStream.readUTF()` under
  the CLDC 1.1 order (readFloat 30, readDouble 31, readUTF 32). RustJava's class
  loader now wraps resource streams in the class named by the
  `rustjava.resource_stream_wrapper` property, which the LGT runtime sets to
  `java/io/DataInputStream`. `java/io/OutputStream` got its CLDC table too
  (close=14 from 메이플스토리2007's record writer).
- **Thread table corrected** (2026-10-01): the slot-32 call 슈퍼액션히어로 makes is
  on that resource stream, not on a Thread, so the earlier CLDC-order Thread
  entries beyond the confirmed start=10/setPriority=14 were dropped; the table
  keeps isAlive=13 as a guess and vtable_size 18.
- **LGT vtables carry 64 slots and missing-slot stubs are shared** (2026-10-01):
  every vtable wie allocates now has room for `VTABLE_CAPACITY` (64) entries, the
  slack filled with missing-entry stubs, and `set_vtable_entries` grows in place
  while the new table fits — instances created before linking appended an entry
  keep dispatching correctly, and a title that calls through a slot wie knows
  nothing about (학교가는길 lr 0x25c0, 훼밀리마트타이쿤 lr 0xc5544) fails with the
  class and slot number instead of `jump to unmapped pc 0x0`. The first cut
  made `make_svc_stub` once per slot per class, which exhausted the 4096-stub SVC
  region during `wie-lgt`'s own JVM test; the resulting fatal error was raised as
  a `net/wie/WieError` exception, defining *that* class failed the same way, and
  the retry recursed until the stack overflowed (macOS crash report:
  `define_class → JavaClassDefinition::new → Jvm::exception → new_class → … →
  define_class`). Three fixes: `ArmCore::shared_svc_stub` hands out one stub per
  `(category, id)` so all missing-slot stubs together cost at most 64; the stub
  region grew to 1 MB (65536 stubs — every Rust-implemented Java method the guest
  can call takes one, so the old 4096 cap was within reach of a large title);
  and `LgtJvmImplementation::define_error` aborts with both errors when a class
  definition fails while the error for an earlier failure is being raised,
  instead of recursing.
  The named stubs then exposed the next layers in one sweep: 학교가는길 called
  `java/util/Stack` slot 32, 훼밀리마트타이쿤 `java/lang/StringBuffer` slot 22, and
  서든어택포켓/턴 (both "no frame painted" before) `java/io/DataOutputStream`
  slot 19, then all three `java/io/ByteArrayOutputStream` slot 16. The CLDC 1.1
  declaration order reproduces every confirmed index in Vector (size 15,
  elementAt 23, removeElementAt 27, insertElementAt 28) and StringBuffer
  (append(Object) 17, append(String) 18, append(I) 23, delete 27), so those
  tables are now complete and Stack (push 32 …), DataOutputStream (writeBoolean 15
  … writeUTF 24) and ByteArrayOutputStream (reset 15, toByteArray 16, size 17)
  follow from them. 학교가는길 now runs its 40 s (one screen so far), the other
  three are next. Found on the way: `set_vtable_entries` must never grow a
  compiler-laid-out vtable in place — it has exactly `vtable_count` slots inside
  the module image — so in-place growth is limited to tables wie allocated on
  the heap.
- **놈3 is nondeterministic, not regressed** (2026-10-01): across identical
  `WIE_VCLOCK` runs it either passes the 이용안내 screen on the first key, sits
  there for 40 s with the paint loop running, or (base build, debug logging)
  stops calling wie altogether after 2 s with the title glyphs garbled. The same
  binary produces both outcomes, so the key-wait depends on something outside
  the virtual clock; finding the remaining source of nondeterminism is the
  prerequisite for debugging it (and the other flaky titles: 삼국지연의2,
  2006독일축구, 동전쌓기2006, 붕어빵타이쿤3's rare thread fault).
  Two sources found the same day, both hash-map iteration order (hashbrown's
  default hasher is seeded per process): `wie-backend`'s executor polled its
  tasks out of a `HashMap`, so which guest thread ran first in a step was a
  coin toss, and RustJava's `collect_garbage` destroyed the unreachable set in
  `HashSet` order, so the allocator's free list — and every later address —
  differed between runs. The executor now keeps tasks in a `BTreeMap` (spawn
  order) and the collector destroys garbage sorted by identity.
- **LGT Java import 0x64 = interface dispatch table** (2026-10-01): with
  ByteArrayOutputStream mapped, 턴's loader thread died with `Unknown lgt java
  import: 0x64`. The module's import thunks are 16-byte entries `{push {lr}; bl
  resolver; table; index}` whose resolver patches the entry into `ldr ip,[pc,#4];
  bx ip` after `get_import_function`, so a thunk address identifies its import.
  The five call sites of the (0x64, 100) thunk all read `[class+8]` (descriptor)
  then `[descriptor+8]` (name) of an interface class and call
  `import100(object, name)`, then index the result with a linked interface
  method index and call `[table + 4*index + 4]` — exactly what older titles do
  through 0x0a, which this compiler no longer emits. So 0x64 shares the
  GetInterfaceDispatchTable handler. (First guess "link class" was wrong: the
  per-class `fn_get_class` stubs call thunks 0x1403288/0x1403298 = (0x64, 11/12)
  = RegisterClass/ResolveClass, not 100.) `get_import_function` now logs the
  guest return address (`from 0x…`), and a missing vtable slot reports its
  caller and r1–r3.
  With 0x64 answered, 턴 called `[table + 4]` and hit `PlayGuide vtable index 0`:
  wie answered an interface dispatch request with the *interface's* vtable (fine
  for wie-defined interfaces, whose entries dispatch by name), but a generated
  class keeps a per-interface reference cell `{ptr_interface_class, target of
  method 0, target of method 1, …}` (PlayGuide: `0x1401334` → IEventHandler +
  five code pointers), and the title indexes that cell directly. The handler now
  walks the receiver's generated class chain for the cell whose class is the
  named interface and falls back to the interface vtable only for wie classes.
- **Collect on heap exhaustion; one guard for every raised wie error**
  (2026-10-01): with deterministic scheduling 붕어빵타이쿤3 aborted at 21 s with a
  host stack overflow. The crash report shows the loop: `Throwable.<init>` →
  `JavaLangString::from_rust_string` → instantiate → allocation fails →
  `jvm.exception(WieError)` → needs a `[C` → `instantiate_array` fails →
  `jvm.exception` → … The guest heap was simply full — the LGT runtime only
  collected inside two WIPI-C resource calls, so a Java title that never calls
  `System.gc()` runs until its 256 MB are gone. `instantiate`/`instantiate_array`
  now collect once on `AllocationFailure` and retry, raise
  `java/lang/OutOfMemoryError` if that still fails, and every site that turns a
  wie error into a Java exception goes through `jvm_support::error::raise`, which
  aborts with both messages when a failure happens while another one is being
  raised (the class-definition guard above is the same helper now).
- **InputStream / DataInputStream gaps** (2026-09-29): InputStream now carries
  the full CLDC order (skip(J)=13 backed by 배틀몬스터's loader thread; mark,
  reset, markSupported 16-18), and DataInputStream 19-22 (readFully x2,
  skipBytes, readBoolean — readFully([B)=19 by 스파이더맨3, readFully([BII)=20 by
  서든어택포켓). RustJava already implements all of them. 스파이더맨3 now plays
  through its title, difficulty select and first stage; 레전드오브마스터 and
  배틀몬스터 run the full 40 seconds.
- **`org.kwis.msp.media.Player.resume(Clip)`** looked up `start(Z)V` on the MIDP
  `Player` interface (which only has `start()V`) instead of `net/wie/SmafPlayer`,
  so 배틀몬스터 died on its first resumed clip.
- **LGT collector robustness + two root-set gaps** (2026-09-29): with keys
  delivered, 일지매영웅전기 and 스파이더맨3 panicked inside `collect_garbage` and
  놈3 did so in 2 of 3 runs (a preemption-timing shift from the changes above
  exposed it). A reference-typed instance word that does not point at a live
  object (header chain instance -> dispatch table -> class record whose vtable is
  that table) is now logged with its holder and read as null, and an object
  whose header no longer reads is leaked with an error instead of being sized
  and freed. That turns the panics into log lines (일지매 9 -> 28 frames) but the
  corruption behind them remains:
  1. **Thread subclass layout overlap.** LGT's `java/lang/Thread` has two
     instance words — 스파이더맨3's `c extends Thread` puts its first own
     reference at word 2 (bitmap `0x3f 0xc0`, MSB-first: words 2-9). wie's
     RustJava Thread occupies words 0-8 (id J, target, name, priority,
     interrupted, started, alive, daemon), so the app's fields and wie's thread
     state overwrite each other (words 6/7 read 1 = started/alive). Any app class
     extending Thread is affected; the fix is to keep wie's Thread state outside
     the two ABI words.
  2. **Guest stack is not a root.** AOT code keeps locals in ARM registers and
     stack; `System.gc()` (일지매 calls it 165 times in 40 s) can free an object
     that is only held there, and the pointer later stored into a field dangles
     (일지매 `atdata/ITEM_PREFIX` word 0). Needs a conservative scan of guest
     stacks as extra roots (a RustJava hook).
  The bitmap bit order itself checks out: `ITEM_PREFIX` has five instance words
  and bitmap `0xf8` — exactly words 0-4 MSB-first, while LSB-first would mark
  words that do not exist.
- **Next layers found while doing the above** (2026-09-29): 당신은골프왕 divides
  by `getSystemProperty("PHONENUMBER").length()`; wie returns "" on purpose
  (see Phone-number DRM below), so it raises `/ by zero` at lr 0x576f.
  슈퍼액션히어로's thread calls slot 32 on an object that is a plain
  `java/lang/Thread` in wie — no CLDC Thread has that slot, so the object in that
  static differs from the handset's; not traced yet.
- **LGT field-import placeholder for wide fields** (2026-09-29): the per-class
  field import tables carry one entry per 32-bit word, so the high word of a
  `long`/`double` field is an entry whose name and descriptor pointers are both
  0 (학교가는길 `an.cB J`). wie read it as a string → `Invalid memory access;
  address: 0` while linking. `link_field_imports` now gives that entry the low
  word's index + 1.
- **Diagnostics** (2026-09-29): the ARM engine logs `jump to unmapped pc … (lr …)`
  and `memory fault accessing … at pc …`; `JavaMethod::run` refuses target 0
  with the method name; missing-vtable stubs already named the class/index;
  `RUST_LOG=wie_lgt=debug` now prints `Registering/Preparing LGT Java class`,
  `Linking public class … @tables`, and preparation errors carry the class name.
  Later the same day: every imported field/virtual/interface member is logged
  (`Imported virtual method …`), a faulting guest call logs the receiver still in
  r0 (`r0 at fault: … (class, N vtable slots)`), and guest-raised divide-by-zero
  logs its call site.
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

- **KTF** — `com.ktf.kfc` (~32 GUI widgets). Started: GForm, GFormBase,
  GMenubarForm (constructor-only, ShellComponent-based — enough for
  미니게임패밀리 to boot to its game-select screen). Remaining: GButton,
  GList, GMenuBar, GTextField, GMsgBox, …, `wec` (~25 hardware: Camera, GPS, AddressBook, SubLCD,
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

Latest full boot run (2026-07-28, 313 games, headless, virtual clock, 8s):
**226 run and paint (72%)** — up from 221 after this change set (놈3/
로스트아일랜드/서울타이쿤2/심시티/추억의달고나 boot deterministically now;
거상(벚꽃단의음모) newly boots; 만귀토벌전 regressed to a reported error
because the p/ fix lets it run further into a null access). Note the
virtual clock undercounts CPU-heavy games against a wall-clock cap: 귀혼무사편
and 어스토니시아 ep1-3 need a larger cap (or real-time mode) to finish
booting and are counted as booting above.

### Flakiness root cause (2026-07-28)

The run-to-run flips (exit 0 ↔ panic) were traced to the real-time clock:
`Executor::tick` budgets stepping by wall-clock (8ms) and sleep wakeups use
`Platform::now()`, so scheduling interleaves differently every run and
machine-load-dependent init races (paint before resources load, JVM entry
before thread registration, class reads during init) surface probabilistically.
Verified with a **virtual clock** (app-repo headless `WIE_VCLOCK=1`: `now()`
advances 1ms per call): previously-flaky 로스트아일랜드/서울타이쿤2/심시티/
추억의달고나 all became 10/10 clean with bit-identical frames. Reclassified
under the virtual clock:

- 놈3 — *deterministic* crash, not flaky. Root-caused (2026-07-28, this
  change set): ① its bundled save probe `/nom` failed because lowercase
  `p/` archive entries were not mounted (only `P/` was trimmed) — fixed, 19
  games bundle lowercase `p/` files; ② the game then calls a String
  constructor through an index-free ARM dispatch that lands on
  `<init>([CII)V` while clearly intending `<init>(Ljava/lang/String;)V`
  (arg0 is a String, the two ints are stale pointers). Passing a non-array
  where an array is declared is no longer wrapped as an array, so this now
  raises a catchable IllegalArgumentException instead of a Rust panic — the
  crash class is gone, but the game still parks on a black screen; the
  mis-dispatch itself is still open. Discovered along the way: the KTF
  method/field name struct's leading byte is not a constant 0 but the low
  byte of the Java string hash of `"descriptor+name"` (verified 17/17
  against real game lookups) — we now write it correctly.
- kbo프로야구/탁재훈신맞고/2010밴쿠버올림픽 — *deterministic* hangs
  (CPU-bound before the first paint), not timing races.
- 크로스워드 — still nondeterministic even under the virtual clock with a
  clean data dir (boot NPE ~50%); a second-order source remains (host hash-map
  iteration order is per-process seeded). Follow-up.

### Progression tiers (T2/T3 re-measurement, 2026-07-28)

All 221 boot-ok games, virtual clock, contact-sheet visual classification.
Initial pass used a 16s key scenario; non-T3 games were re-measured with a
30s scenario (OK×5 spread 4–16s, DOWN@19 OK@22 5@25) after discovering the
16s window undershoots slow boot sequences (carrier notices + logo chains) —
that alone reclassified 33 games upward. Final:

- **T3 (menu navigation or beyond): 159** — 34 visibly reach gameplay
  (updated after the p/-mount/name-tag/non-array change set: 거상, 로스트
  아일랜드, 서울타이쿤2, 심시티, 추억의달고나, 크로스워드 joined T3; 심시티/
  달고나/크로스워드 reach gameplay).
- **T2 (responds but stuck): 23** • **T0 (no input response): 26** •
  **blank screen: 18** • boot error: 1 (만귀토벌전)
- 크로스워드 still boots only ~50% of the time (host hash-order dependent,
  open) but plays the puzzle when it does.
- By carrier: KTF 128/158 T3, LGT 18/45, SKT 13/19.

Re-measured 2026-08-04 after the callSerially-delay and LWC/media-method
fixes (full boot batch + 30s re-scan of the non-T3 games, no demotions):

- **T3: 172 (+13)** • **T2: 24** • **T0: 17** • **blank: 13**. By carrier:
  KTF 137/164, LGT 22/45, SKT 13/19.
- Beyond the games fixed by name earlier this session, the re-scan caught
  side-effect unblocks (callSerially/LWC reached loops we hadn't retried):
  아르덴전기, 화장빨인생, 던파귀검사편, 리듬스타2, 메이플스토리_도적편,
  아니마 all T2 → T3.
- Boot count is ~flat: 뮤_흑기사편 newly boots; 슈렉3 now progresses to its
  logo (callSerially) then crashes on the `address 0` bucket instead of
  freezing; the CPU-heavy games (귀혼무사편, 어스토니시아 ep1-3) and 크로스워드
  are virtual-clock wall-cap / boot-flaky artifacts, not regressions.

Remaining non-T3 buckets (from the 30s sheets):

- **Gamevil "empty dialog" cluster** (제노니아1/2, 하이브리드2, 놈ZERO):
  native WIPI-C (Clet) games behind LGT's thin `CletWrapperCard` Java shim.
  Re-investigated 2026-07-29; earlier "proprietary SVC 2000 / network-gated"
  notes were both **wrong** and are retracted:
  - No unknown SVC in normal runs. The "SVC 2000" only appeared after an
    experiment that faked `MC_netConnect` success, sending the game down a
    path with an uninitialized socket handle; the garbage r12 (SVC id is read
    from IP) surfaced as a bogus number — an experiment artifact.
  - **Not network-gated.** With no injected input the game makes *zero* net
    calls; `MC_netConnect` only fires as a reaction to pressing OK on the
    dialog. So the resting state is reached without any networking.
  Actual observed state: resources load cleanly (fonts .ft2, particles .ptc,
  UI .mpl/.pzx — no NOENT), the main loop runs (setTimer ~470/run), and every
  frame the game redraws one dialog: a light-gray panel (`FillRect` 0xdefb),
  white/gray beveled border, an **empty** dark inner box (`FillRect` 0x3186,
  uniform — verified by inverting the fill: zero content pixels), and an
  **unlabeled** orange button (`FillRect` 0xf580 + bevel lines). Crucially
  there are **no glyph draws anywhere** — the game renders all chrome via
  DrawLine/FillRect but never emits the dialog's message text or button label.
  So the text content is simply absent from the game's own state, not lost in
  our text path. Dead ends ruled out (2026-08-01): the game probes for
  `com/MainUI.mpl`/`GameUI.mpl`/`Title.mpl` which the jar ships only as `.gui`
  (different magic — `.gui` `01..`, `.mpl` `30061a..`, `.pzx` `PZX`), gets
  NOENT, and continues fine — this is normal optional-resource probing, not
  the cause (a `.mpl`→`.gui` fallback served nothing and changed nothing). The
  dialog chrome is drawn procedurally, not from a UI template. Pinning why the
  game emits no glyphs needs ARM/bytecode stepping of its draw path — a large
  dedicated task; do not re-chase the missing-`.mpl` lead.
  - **[2026-08-11] CONFIRMED — the "no glyphs" symptom was the GetFramebufferBpp
    render-path bug (fixed below), NOT a missing text path.** The earlier
    conclusion ("the game emits no glyphs") was wrong: the glyphs *were* being
    drawn via the game's own custom 32-bit blit, which the draw-call
    instrumentation didn't count and which wrote garbage into our 16-bit
    framebuffer. With the bpp fix, **제노니아1, 놈ZERO, and 하이브리드2 all now
    render their full Korean text** (the "이용안내 … 아무키나 누르세요" standalone-game
    notice) instead of an empty/garbled dialog. So the whole cluster's rendering
    is resolved by one 6-line fix. The remaining blocker is a different one: these
    are network-featured games (ranking/mail/item-gift over `MC_netConnect`) but
    are **단독형 (standalone) — playable offline**; pressing a key past the notice
    reaches a real main menu (이어하기/새게임), and 새게임 → "해당 슬롯에 데이터가
    없습니다, 새로 시작하시겠습니까 [예/아니오]". Confirming 새 게임 then crashes in the
    save-data-init path: an accessor object's `this+0x8` field holds a wild pointer
    (`0xfffe0808`) — set at game pc `0x35fcc` from the return of game fn `0xa590`,
    which returns a stale/corrupt entry from a per-index accessor table
    (`this+0x10`, 8-byte stride) rather than a fresh alloc. Root cause not yet
    pinned (a level deeper than the bpp fix); this is the next lead for the cluster.
    Do NOT re-chase the missing-`.mpl`/no-glyphs leads.
- **테일즈위버 막시민편** (LGT Clet — investigated 2026-08-24): **plays fine, no
  emu bug.** Renders perfectly (intro notice, title "press any key", main menu, the
  바이오리듬 offline feature). The user's reported "408 오류" is the game's own
  **"서버 접속에 실패했습니다. 재접속 또는 문의처로 연락주세요"** dialog on "게임시작":
  our `MC_netConnect` returns `M_E_ERROR`, so the first connect attempt shows this
  dialog. **But it is NOT a hard block** — dismissing/retrying the connect a few
  times (netConnect fired ~3× in a headless run before it advanced) lets the game
  fall through to offline character creation: **스타일 선택 (물리복합형/베기형/마검사형)
  → in-game town map with NPCs → story cutscene (르베리에 dialog)**. Full offline
  gameplay works; only online extras (ranking/mail) need the dead server. (An
  earlier note here claiming "not fixable without a server" was wrong — corrected
  after the user pointed out that retrying works.) Faking a *success* callback is
  the wrong fix: it pushes the game into the socket protocol and dies with
  `Unknown LGT WIPIC SVC id 2000` (제노니아's uninitialized-socket artifact) — the
  M_E_ERROR + retry path is the one that actually reaches gameplay. Possible future
  UX polish: make the connect fail fast so fewer manual dialog dismissals are needed.
- **`MC_grpGetFrameBufferBpp` returning 0 for a stale handle** (fixed 2026-08-06,
  리듬페스티발): the API read the passed framebuffer handle and returned its `bpp`
  verbatim. Some Clets pass a stale argument register here (real handsets treat
  pixel depth as a fixed screen property independent of the argument), so the
  handle resolves to zeroes and the API returned 0. A game then compares the
  screen depth against 16, picks a **32-bit** render path, and blits 32-bit
  pixels into our 16-bit framebuffer — garbling every sprite/glyph *and*
  overflowing the draw buffer into the adjacent heap (this is the actual source
  of the `getNextEvent` "Allocation failure" for these games, upstream of the
  corruption-resilient heap walk that only contained it). Fix: fall back to the
  framebuffer depth (16) when the handle doesn't resolve to a real framebuffer.
  리듬페스티발 now renders text/sprites and its heap self-overflow disappears
  (header repairs drop from firing every frame to zero).
- **Integrity/re-download notices** (2008베이징올림픽, 레이카르나, …): the
  game itself decides it is corrupted and parks on a "download again" screen.
- **Network-consent popups** (리듬스타2, 이터널사가3, …): stuck on a
  connect-confirmation dialog the standard scenario cannot answer.
- **Key-ignoring notice screens** (에바스토 etc.): keys verified delivered to
  the clet (`CletWrapperCard.keyNotify`), game still waits on something else.

### callSerially(Runnable, delay) frozen-loop fix (2026-08-01)

Sweeping the T0 ("no input response") and BLACK buckets found a shared cause:
`org.kwis.msp.lcdui.Display.callSerially(Runnable, int)` was a no-op stub that
dropped the runnable. Several games bootstrap their entire game loop with a
single `callSerially(runnable, delay)` call, so dropping it froze them on the
first painted frame (looked like T0) or before first paint (BLACK). Fixed by
enqueuing the runnable on the event loop like the no-timeout overload (the
delay is ignored — we have no delayed scheduler here — which is fine for loops
that re-schedule each frame). All 6 games that hit the stub now advance:
광수의똥/데빌헌터/푸시푸시삼국지/피자타이쿤 reach full menu navigation (T3,
피자타이쿤 was BLACK), 리얼사커2007 reaches its title/loading, 슈렉3 gets to
its logo then hits a separate null-access. 31-game regression unchanged.

### Missing method fills (2026-08-01)

Aggregating "Method X not found" fatals across the boot batch showed each is
one game, but several are standard classes we simply hadn't registered.
Filled them (safe/minimal bodies; missing them aborted the whole app):
- `org.kwis.msp.media.BaseClip.setBuffer([BI)Z` — feed the buffer as the
  clip's SMAF data like putData. 미궁미술관살인사건 BLACK → T3 (in-game story).
- `org.kwis.msp.lwc.Component.repaint()V`/`(IIII)V`, `getX/getY/getWidth()I`;
  `ContainerComponent.validate()V`; `AnnunciatorComponent.layout()V` — LWC
  widget geometry/paint isn't wired to the framebuffer, so these are no-ops
  /zeros, but registering them lets LWC apps run. 질풍노도17대1 BLACK → T3
  (story), 뮤_흑기사편 boot-error → T3 (story). 해적왕2007 gets past the LWC
  cascade but then hits "jump native address is null" (same open bucket as
  주타이쿤2). LWC is still non-rendering; these just stop the aborts.
- `org.kwis.msp.media.Player.resume(Clip)Z` — mirror the existing Clip
  play/stop overloads (start the clip's player). Clears the crash in
  미니게임파티. 31-game regression unchanged; clippy clean.

### "jump native address is null" cluster — tolerance rejected (2026-08-04)

~6-7 games (다크슬레이어2, 삼국장군전, 마스터오브소드2, 맞고삼국대전,
미니러비, 주타이쿤2, 해적왕2007) abort in `interface.rs` when a
`java_jump_*`/`call_native` trampoline gets `address == 0` — the game
dispatches to a method whose native body pointer resolved to 0. This is a
shared *symptom*, not one cause: the triggers differ (a `DataBaseRecordException`
recovery path after a missing save in 주타이쿤2; `Class.forName`/
`ClassLoader.loadClass` reflection in 다크슬레이어2/삼국장군전/마스터오브소드2;
`Object.wait` in 미니러비; a bare `<init>` in 맞고삼국대전). Making the null
jump a no-op returning 0 was tried and **reverted** — it is load-bearing:
주타이쿤2 froze static, 미니러비 started panicking, and the reflection games
just moved to NPE/`Invalid memory access; address: 0`. The real fix needs
KTF-dispatch RE (why `get_java_method`/vtable resolves a method whose
`fn_body` is 0 on these paths — likely related to the 놈3 index-dispatch
mystery). Do not re-try the tolerance approach.

### In-play crash sweep (2026-08-04)

Aggregating crashes during the 30s input scenario:
- **`org.kwis.msp.lcdui.Jlet.getCurrentJlet()`** was missing (only `getActiveJlet`
  existed) — added as the same impl (returns the `currentJlet` static). Clears
  대박투어타이쿤's crash; it now runs the full scenario.
- **`Allocation failure at net/wie/EventQueue.getNextEvent`** (리듬페스티발,
  메이플스토리_도적편, and any game that plays long enough): root-caused
  2026-08-05 with allocator/heap instrumentation (all reverted). It is **not a
  leak** — it is **heap-header corruption**:
  - Not a Java-object leak: with a periodic GC the live-object set is bounded
    (~229 objects) yet the OOM still fires.
  - Not a raw-alloc leak: instrumenting `Allocator::alloc/free` showed only
    ~1.5 MB net-allocated at the failure.
  - Walking the `ListAllocator` heap at the failure found a block whose header
    word had been overwritten with `0x0000ffff` (a white RGB565 pixel): its
    `size()` becomes `0xffff` (65535, not 4-aligned), so the free-list walk
    lands mid-word, reads garbage (`0x7bfffdff`, in_use=true, ~2 GB) as the next
    header, and can no longer reach the free space past it — every later alloc
    that needs the list region fails.
  - Fully pinned with a guest-store watchpoint (2026-08-05, all reverted): the
    corrupting store is the **game's own blit at `pc=0x1aee2`** (deterministic
    under the virtual clock) doing 32-bit stores of RGB565 pixels
    (0xce9f…0xffff) that run **4 bytes past the end of a 31417-byte buffer**
    (user 0x4014b244, block ends at 0x40152d04) into the next block's header.
    The buffer is odd-sized → a game `MC_knlAlloc`, not one of our (always
    even) image/framebuffer buffers. So this is the **game overflowing its own
    heap buffer**; it ran on real handsets because their allocator left slack
    after the block, whereas our `ListAllocator` packs a header+canary
    immediately after. NOT `FrameBuffer::write` (0 oversize write-backs).
  - A trailing-guard-slack experiment (256 B after every block) was **rejected**:
    it removed the Allocation-failure but shifted the whole heap layout, so the
    game corrupted a different header and crashed at startup with "Invalid
    allocation header" instead. Do NOT re-try GC or naive guard padding.
  - **Fixed 2026-08-05 with a corruption-resilient heap walk** (`ListAllocator::
    find_address`): every well-formed block size is a multiple of 4, non-zero,
    and in-bounds, so a header that fails `is_plausible_size` has been scribbled
    over (its size and in-use bit are both just pixel bytes — the in-use bit is
    whatever colour the pixel was, so it can't be trusted either). Such a header
    is rebuilt as a *free* block spanning up to the next in-use block — located
    by a canary-validated forward scan (`find_next_inuse_boundary`) — or the heap
    end, and the repaired header is written back so later walks stay in sync. The
    canary scan preserves any genuine in-use block after the corruption; the
    block at the corrupt header is unrecoverable anyway (the old walk crashed on
    it), so reclaiming it as free is strictly more resilient. Verified: 리듬페스티발
    now runs the full 60 s scenario (9 header repairs, 0 Allocation-failure)
    instead of crashing ~10 s in; unit tests cover the free-tail, in-use-bit, and
    stop-at-in-use cases. The repair is a one-time cost per corruption (the header
    is rewritten valid), and in practice the scan finds a nearby in-use block so
    it stays cheap.
- **KTF DB record-info + GetContext filled in** (2026-08-26, user report): an app
  user's screenshot showed 데몬헌터 dying at startup with
  `Unimplemented: 10: MC_dbGetNumberOfRecords`. Implemented
  `MC_dbGetNumberOfRecords` / `MC_dbGetRecordSize` for KTF's single-record
  stream-handle model (count = mirror non-empty ? 1 : 0; size = mirror length —
  games call these right after open to tell "is there a save?"), and wired KTF's
  `MC_grpGetContext` slot to the existing `graphics::get_context` impl (it was a
  stub; LGT already used the real one — this was also 에픽크로니클2's crash). A
  KTF-wide 8s sweep found 데몬헌터 as the only boot-time caller. 데몬헌터 now boots
  past both layers and renders its notice dialog, then hits the known `address 0`
  bucket below (separate deep issue).
- **데몬헌터 fully playable** (2026-08-26, follow-up to the record-info fix): four
  more layers, each pinned by disassembling client.bin (loaded at IMAGE_BASE
  0x100000) against runtime traces:
  - The input-triggered `address 0` crash was the game dereferencing the result
    of `MC_dbOpenDataBase("Config.dat", mode 1)` without checking for -12. KTF's
    open treats a missing DB as an openable empty one: added
    `open_database_ktf` (mode-1 open of a missing DB returns an empty handle
    instead of M_E_NOENT; LGT keeps the strict behaviour — 제노니아 probes with
    open and handles the -12).
  - `read_packaged_database` only looked at jar classpath resources, but KTF
    archives ship preloaded DBs as top-level `P/<name>` files mounted on the
    virtual filesystem — added a filesystem fallback so Config.dat/Save0.dat
    etc. are visible as packaged databases.
  - `MC_dbExists` (KTF slot 16) actually returns the WIPI-standard 0/-12, not a
    1/0 boolean (the old guess made 데몬헌터 loop on its "restart the app"
    screen forever), takes an out pointer it fills with `{0, 0, record size}`
    (word 2 feeds the game's buffer allocation directly — guarded to only write
    when the argument plausibly points at memory, since boot-time call sites
    pass small flags there), and considers packaged databases.
  - `MC_dbGetNumberOfRecords` takes a database *name*, not a handle (데몬헌터
    asks about "Patch"); reimplemented name-based with a handle-magic sniff.
  - `MC_grpGetContext`: the third argument is an out pointer — struct-valued
    attributes (ClipIdx/OffsetIdx) write the struct through it. The game polls
    the clip every frame during stage loading and never finished while the op
    was unsupported. (Superseded by upstream's implementation in the 2026-09
    sync, which writes every attribute through the out pointer.)
  Result: boots → title ("PRESS 이어하기 KEY") → stage load ("루베르전초기지") →
  in-game play with HP/MP HUD and a moving player character. 31-game
  regression unchanged.
- Still-open buckets hit here: `address 0` family (보글보글, 화장빨인생, 놈ZERO,
  하이브리드 — see the jump-native cluster above), `Invalid allocation header`
  (LGT_KBO프로야구2009), an ambiguous high `LGT WIPIC SVC id 901` (슈퍼액션히어로3,
  likely a garbage dispatch like the 2000 case — not mapped). 미니게임씨네마's
  missing timer method (recorded here earlier as `TimerTask.cancel()Z`) was
  actually `java.util.Timer.cancel()V`; both it and `TimerTask.cancel()Z` are now
  implemented RustJava-side (Timer.cancel terminates the timer thread + clears
  pending tasks), so 미니게임씨네마 runs the full scenario.
- Input-triggered crashes: 일지매_영웅전기/현영맞고_2006 panic after menu
  entry; LGT_KBO프로야구2009 corrupts the allocator on first keypress
  ("Invalid allocation header"). SKVM `WieAudioClip.close` double-close panic
  (노리타이쿤, 더팜1, 드래곤나이트EX) fixed in this change set — 노리타이쿤/
  드래곤나이트EX now reach T3.

Keep this document updated in the same change set as the implementation work
it describes.
