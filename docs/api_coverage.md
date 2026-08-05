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
    allocation header" instead. A clean fix would need to replicate the LGT
    handset allocator's block layout/rounding (unknown), or make the heap walk
    resilient to a corrupt header (risky) — both large. Do NOT re-try GC or
    naive guard padding.
- Still-open buckets hit here: `address 0` family (보글보글, 화장빨인생, 놈ZERO,
  하이브리드 — see the jump-native cluster above), `Invalid allocation header`
  (LGT_KBO프로야구2009), an ambiguous high `LGT WIPIC SVC id 901` (슈퍼액션히어로3,
  likely a garbage dispatch like the 2000 case — not mapped), and
  `java.util.TimerTask.cancel()Z` missing (미니게임씨네마 — RustJava-side).
- Input-triggered crashes: 일지매_영웅전기/현영맞고_2006 panic after menu
  entry; LGT_KBO프로야구2009 corrupts the allocator on first keypress
  ("Invalid allocation header"). SKVM `WieAudioClip.close` double-close panic
  (노리타이쿤, 더팜1, 드래곤나이트EX) fixed in this change set — 노리타이쿤/
  드래곤나이트EX now reach T3.

Keep this document updated in the same change set as the implementation work
it describes.
