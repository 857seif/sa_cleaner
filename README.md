# SA Cleaner v2.0

Windows memory cleaner. **Rust UI** + **C++ engine** (static lib).  
Elevated app that purges standby/working-set/cache and optional light disk junk.

---

## What it does

| Action | Mechanism |
|--------|-----------|
| Working Set | `NtSetSystemInformation` + EmptyWorkingSet |
| System Cache | File-cache info class + empty WS companion |
| Standby List | Purge standby (largest typical reclaim) |
| Modified List | Flush modified pages |
| Registry | Registry hive reduce class |
| Combine | Page combining class |
| Low Priority | Purge low-priority standby only |
| Flush Volumes | `FlushFileBuffers` on `\\.\X:` fixed drives |
| Clipboard | `EmptyClipboard` |
| DNS | Hidden `ipconfig /flushdns` |
| Temp | Non-recursive delete under user/system temp |
| Prefetch | Delete `%SystemRoot%\Prefetch\*` |
| Thumbnails | Delete `thumbcache_*.db` under Explorer folder |

UI runs **clean on a background thread** and posts results back — window does not freeze.  
Minimize / **To Tray** keeps **auto-clean** alive in the background.

---

## Requirements

- Windows 10/11 x64
- Administrator (UAC prompt built-in)
- Build: Rust + C++ toolchain (MinGW `x86_64-pc-windows-gnu` or MSVC)

---

## Build

```bat
cd sa_cleaner_v2
build.bat
```

Linux cross (MinGW):

```bash
chmod +x build.sh
./build.sh
```

`build.rs` compiles `src/cpp/engine.cpp` with `cc` and links:

`advapi32 kernel32 gdi32 user32 shell32 psapi ole32`

Release flags: LTO, strip, `panic = abort`.

Output: `target/release/sa_cleaner.exe` (copied to `sa_cleaner_v2.exe` by the scripts).

---

## Usage

1. Run EXE → accept UAC.
2. Check the cleaners you want (or **Safe Profile**).
3. Optional: set **Threshold %** (1–99), click **Start Auto**.
4. **CLEAN NOW** for a one-shot pass.
5. **To Tray** or minimize — auto-clean continues; double-click tray icon to restore.

**Fast RAM-only:** Working Set + System Cache + Standby + Modified.  
Leave Temp/Prefetch/Thumb off unless you want disk scrubbing.

**Aggressive:** shorter waits + second standby pass.

---

## Repository layout

```text
sa_cleaner_v2/
├── Cargo.toml
├── build.rs
├── build.bat
├── build.sh
├── CHANGELOG_v2.md
├── README.md
└── src/
    ├── main.rs              # Win32 UI, tray, async clean
    └── cpp/
        ├── engine.h         # C API
        └── engine.cpp       # Nt + file operations
```

---

## Architecture

```text
┌─────────────────────────────────────┐
│ main.rs  (windows_subsystem)        │
│  • window / controls / theme        │
│  • tray icon                        │
│  • std::thread → sa_clean           │
│  • PostMessageW(WM_APP_CLEAN_DONE)  │
└─────────────────┬───────────────────┘
                  │ extern "C"
┌─────────────────▼───────────────────┐
│ engine.cpp (static lib)             │
│  • privilege adjust                 │
│  • parallel CreateThread workers    │
│  • NtSet/QuerySystemInformation     │
│  • temp/prefetch/thumb/dns/clip     │
│  • auto-clean thread + critical sec │
└─────────────────────────────────────┘
```

---

## C API (`engine.h`)

```c
#define REDUCT_WORKING_SET      0x01
#define REDUCT_SYSTEM_CACHE     0x02
#define REDUCT_STANDBY_LIST     0x04
#define REDUCT_MODIFIED_LIST    0x08
#define REDUCT_REGISTRY         0x10
#define REDUCT_COMBINE          0x20
#define REDUCT_LOW_PRIORITY     0x40
#define REDUCT_FLUSH_VOLUMES    0x80
#define REDUCT_CLIPBOARD        0x100
#define REDUCT_DNS_CACHE        0x200
#define REDUCT_TEMP_FILES       0x400
#define REDUCT_PREFETCH         0x800
#define REDUCT_THUMBNAIL_CACHE  0x1000
#define REDUCT_ALL_SAFE         0x1FFF

int  sa_clean(unsigned int mask);
void sa_get_err(char* buf, unsigned int len);
void sa_get_mem(unsigned long long* total, unsigned long long* used, unsigned long long* free);
int  sa_is_admin(void);
void sa_uac_restart(void);
void sa_set_priority_low(void);
void sa_set_auto_threshold(unsigned int percent); /* 0 = stop */
void sa_stop_auto_clean(void);
unsigned long long sa_estimate_clean(unsigned int mask);
void sa_estimate_each(unsigned long long* out13);
void sa_get_detailed_mem(unsigned long long* total, unsigned long long* avail,
                         unsigned long long* standby, unsigned long long* modified,
                         unsigned long long* cached);
int  sa_get_process_count(void);
void sa_set_aggressive(int enabled);
unsigned int sa_get_last_cleaned_mb(void);
```

Return from `sa_clean`:

- `-1` — not admin
- `0` — all selected ops OK
- `>0` — number of failed ops (partial)

---

## Engine internals (`engine.cpp`)

### Privileges

```cpp
en_priv(L"SeProfileSingleProcessPrivilege");
en_priv(L"SeIncreaseQuotaPrivilege");
en_priv(L"SeDebugPrivilege");
```

### NT glue

```cpp
typedef NTSTATUS (WINAPI *NtSetSysInfoPtr)(SYS_INFO_CLASS, PVOID, ULONG);
// GetProcAddress(ntdll, "NtSetSystemInformation")
```

| Class | ID | Use |
|-------|---:|-----|
| SysFileCache | 21 | Trim system file cache |
| SysMemList | 80 | Empty WS / purge standby / flush modified / low prio |
| SysRegRec | 166 | Registry resident reduce |
| SysCombine | 174 | Page combine |

Commands on SysMemList: `EmptyWS=1`, `FlushMod=2`, `PurgeStandby=3`, `PurgeLowPrio=4`, `PurgeStandbyPriority=5`.

### Parallel workers

Each selected bit → `CreateThread`. Join:

```cpp
WaitForMultipleObjects((DWORD)tcount, threads, TRUE,
                       g_aggressive ? 1500 : 2500);
```

### Standby measurement

```cpp
// SYSTEM_MEMORY_LIST_INFORMATION
// bytes = sum(PageCountByPriority[0..7]) * 4096
```

### Free delta

```text
before = GlobalMemoryStatusEx.ullAvailPhys
run cleaners
after  = ullAvailPhys
g_last_cleaned_mb = max(0, after - before) / 1048576
```

### Auto-clean

```cpp
while (g_auto_running) {
    Sleep(g_aggressive ? 2000 : 5000);
    // if used% >= g_auto_threshold → sa_clean(REDUCT_ALL_SAFE)
}
```

`CRITICAL_SECTION` prevents overlap with manual clean.

### Temp policy

```cpp
delete_files_in_dir(path, /*recursive=*/0);
```

Shallow only — deep walks froze the old builds.

---

## UI (`main.rs`)

### Window chrome

- Class name: `SACleanerV2Dark`
- Style: fixed frame (caption + sysmenu + minimize only)
- Maximize blocked in `WM_SYSCOMMAND`

### Async clean path

```rust
std::thread::spawn(move || {
    let res = unsafe { sa_clean(mask) };
    unsafe { PostMessageW(hwnd_copy, WM_APP_CLEAN_DONE, res as WPARAM, 0); }
});
```

`G_CLEANING` stops a second job while one is running.

### Estimate line

```text
Total ~1.14 GB   [WS 160.0 MB | Cache 220.0 MB | Standby 780.0 MB]
```

Built from `sa_estimate_each` for checked flags only. Heuristic, not a promise.

### Theme

| Token | Dark | Light |
|-------|------|-------|
| Background | `#000000` | `#F5F5F5` |
| Text | `#F0F0F0` | `#111111` |

**BG Shade** rotates near-black backgrounds while dark mode is on.

### Tray

```text
Shell_NotifyIconW(NIM_ADD / NIM_DELETE)
tip = "SA Cleaner — running in background"
```

Minimize or **To Tray** hides the window. Tray click restores. Icon removed on destroy.

### Help strip

Clicking a control fills the bottom label via `help_for(id)` (English, one paragraph per control).

---

## Safe Profile

```text
0x01 | 0x02 | 0x04 | 0x08 | 0x100 | 0x200
WS + Cache + Standby + Modified + Clipboard + DNS
```

---

## Performance

| Scenario | Ballpark |
|----------|----------|
| Safe RAM-only | 1–3 seconds |
| Aggressive RAM-only | 1–2 seconds |
| With Temp/Prefetch/Thumb | disk-bound; still join-timeout capped |

UI message loop keeps running during clean. Free RAM often drops again later as the cache refills — normal Windows behavior.

---

## vs v1

See `CHANGELOG_v2.md`.

Short list: live standby numbers, parallel joins, non-blocking UI, tray auto-clean, true black theme, per-item estimates, shallow temp, no old branding.

---

## Historical multi-language notes (not in v2 binary)

| Language | Original idea |
|----------|----------------|
| Go | Small local HTTP API around mask/mem/auto |
| Nim | Alternate GUI prototype |
| Zig | Optional plugin DLL ABI |
| Lua | Defaults/config script |
| Python | Dev build driver |

v2 ships **only** Rust + C++.

---

## Troubleshooting

| Symptom | What to do |
|---------|------------|
| Admin Required | Approve UAC; check group policy if `runas` blocked |
| 0 MB freed | Standby already empty; nothing to reclaim |
| Partial | One Nt class failed on this OS build; others still applied |
| Ghost tray icon | Process killed hard; restart Explorer |
| Feels slow | Turn off Temp/Prefetch/Thumb; Safe + Aggressive |

---

## Intent

Personal / educational systems tool. Know what standby and working-set trims imply before leaving aggressive auto-clean on shared machines.


---

## Deep dive: Windows memory lists

Windows keeps several lists of physical pages. SA Cleaner only touches the ones exposed through the memory-list system information class and a few user-mode side channels.

### Zero list
Pages already zeroed and ready for fault-in. Not purged by this tool.

### Free list
Unused pages. Increasing free is what the UI reports as “freed”.

### Standby list
Clean pages still associated with a file or pagefile section. Soft-faultable. Purging standby is the main win for “memory cleaners”.

### Modified list
Dirty pages waiting for the modified page writer. Flushing them costs disk I/O.

### Modified-no-write
Special dirty pages not written to pagefile. Rarely large.

### Bad list
Hardware-marked bad pages. Never touched.

Priority sub-bands (0–7) exist inside standby. Low-priority purge targets colder bands first.

---

## Deep dive: Working set trimming

A process working set is the set of pages currently resident for that process. EmptyWorkingSet-style operations ask the kernel to shed pages that can be faulted back later. Effects:

- Idle background apps shrink.
- Active foreground apps may fault pages back immediately (short CPU/disk spike).
- Does not terminate processes.

SA Cleaner issues this via SysMemList / EmptyWS rather than walking every PID with `EmptyWorkingSet`, which is slower and noisier.

---

## Deep dive: System file cache

The cache manager maps file data into kernel address space. On systems with heavy file I/O, cache can hold gigabytes. Trimming cache increases free RAM but can slow the next cold read of those files.

v2 estimate uses roughly half of measured cache size as a soft expectation — actual reclaim varies with how pinned VACBs are.

---

## Bitmask cookbook

```text
RAM-only aggressive:
  0x01 | 0x02 | 0x04 | 0x08 | 0x40   (WS, Cache, Standby, Modified, LowPrio)

Safe default:
  0x01 | 0x02 | 0x04 | 0x08 | 0x100 | 0x200

Disk maintenance add-on:
  | 0x400 | 0x800 | 0x1000          (Temp, Prefetch, Thumb)

Everything:
  0x1FFF
```

Compose masks in code:

```c
unsigned int mask = REDUCT_WORKING_SET | REDUCT_STANDBY_LIST | REDUCT_SYSTEM_CACHE;
int rc = sa_clean(mask);
```

```rust
let mask = 0x01 | 0x02 | 0x04;
std::thread::spawn(move || unsafe { sa_clean(mask) });
```

---

## Control flow: manual clean (sequence)

1. User clicks CLEAN NOW.
2. UI reads checkbox state → `get_mask()`.
3. If mask==0 → warning, return.
4. If `G_CLEANING` → status “Already cleaning”, return.
5. `G_CLEANING = true`, status line updates.
6. Worker thread calls `sa_clean(mask)`.
7. Engine enters critical section.
8. Admin check; fail → -1.
9. Enable privileges.
10. Snapshot avail RAM.
11. Spawn one thread per bit.
12. `WaitForMultipleObjects` up to 1.5–2.5s.
13. Optional volume flush on UI thread side of engine (same call stack).
14. Aggressive optional second standby/WS pass.
15. Snapshot avail RAM again; compute MB delta.
16. Format `g_err`.
17. Leave critical section; return code.
18. Worker `PostMessageW(WM_APP_CLEAN_DONE, res, 0)`.
19. UI handler reads err string + last freed; MessageBox; refresh labels; `G_CLEANING = false`.

---

## Control flow: auto-clean

1. User sets percent in edit box, Start Auto.
2. UI parses 1..99, calls `sa_set_auto_threshold(pct)`.
3. Engine starts `AutoCleanThread` if not running.
4. Loop: sleep → `sa_get_mem` → if used% >= threshold → `sa_clean(REDUCT_ALL_SAFE)`.
5. Stop Auto → threshold 0 → thread exits → handle closed.

While tray-hidden, this loop is the entire residency reason for the process.

---

## FFI rules of thumb

- All engine entry points are `extern "C"`.
- Strings returned into caller buffers are narrow (`char*`), UTF-8 not guaranteed; status text is ASCII today.
- `sa_get_err` always null-terminates when `len > 0`.
- Pointer outs for mem counters must not be null.
- `sa_estimate_each` expects at least 13 `unsigned long long` slots.

Rust side marks every call `unsafe` and keeps buffer lengths explicit.

---

## Privilege failure modes

If `AdjustTokenPrivileges` does not enable a privilege (policy stripped):

- Some Nt memory-list commands return status failure.
- Clean returns partial (>0).
- Others may still succeed (clipboard, DNS, temp).

Always test on a local admin account first.

---

## MinGW vs MSVC notes

| Topic | MinGW | MSVC |
|-------|-------|------|
| `#pragma comment(lib,...)` | ignored (warning) | works |
| Process name API | `GetModuleBaseNameW` | either API OK |
| Link CRT | `-static-libgcc -static-libstdc++` | `/MT` optional |
| `cc` crate | needs g++ in PATH | needs cl |

v2 avoids MSVC-only APIs so GNU builds work.

---

## UI layout map (approx coordinates)

```text
Y 10   Title
Y 50   RAM summary
Y 72   Standby / Modified / Cache
Y 94   Processes + Last freed
Y 128  Checkbox row 1
Y 160  Checkbox row 2
Y 192  Checkbox row 3
Y 224  Thumbnail checkbox
Y 270  Main buttons (Clean, Safe, All, Aggressive, Theme, Tray)
Y 320  Auto label
Y 348  Start / Stop / BG / Threshold
Y 395  Estimate
Y 435  Status
Y 458  Help strip
```

Window client ~760×640, non-resizable chrome.

---

## Message constants used

```text
WM_CREATE          0x0001
WM_DESTROY         0x0002
WM_SIZE            0x0005
WM_SYSCOMMAND      0x0112
WM_COMMAND         0x0111
WM_TIMER           0x0113
WM_ERASEBKGND      0x0014
WM_CTLCOLORSTATIC  0x0138
WM_CTLCOLORBTN     0x0135
WM_TRAYICON        0x8001  (custom)
WM_APP_CLEAN_DONE  0x8002  (custom)
```

---

## IDC assignments

```text
100 Title
101 Mem info
102 Standby line
103 Status
104 Estimate
201-213 Checkboxes (WS..Flush)
301 Clean
302 Start Auto
303 Stop Auto
304 Aggressive
305 Select All
306 Safe Profile
307 Theme
308 BG Shade
309 To Tray
401 Auto label
402 Process count
403 Last freed
404 Help
405 Threshold edit
406 Threshold label
```

---

## Estimating code path

```rust
fn update_estimate(hwnd: HWND) {
    let mask = get_mask(hwnd);
    let mut each = [0u64; 13];
    unsafe { sa_estimate_each(each.as_mut_ptr()); }
    // for each bit set → push "Name <fmt_bytes>"
    // total sum → "Total ~..."
}
```

```cpp
void sa_estimate_each(unsigned long long* out13) {
    // query detailed mem once
    out13[0] = total / 50;      // WS
    out13[1] = cached / 2;      // cache
    out13[2] = standby;         // standby
    out13[3] = modified / 2;    // modified
    // ... fixed small constants for the rest
}
```

---

## Tray implementation sketch

```rust
unsafe fn tray_add(hwnd: HWND) {
    let mut nid: NOTIFYICONDATAW = zeroed();
    nid.cb_size = size_of::<NOTIFYICONDATAW>() as u32;
    nid.h_wnd = hwnd;
    nid.u_id = 1;
    nid.u_flags = NIF_MESSAGE | NIF_ICON | NIF_TIP;
    nid.u_callback_message = WM_TRAYICON;
    nid.h_icon = LoadIconW(0, IDI_APPLICATION);
    // copy tip UTF-16
    Shell_NotifyIconW(NIM_ADD, &mut nid);
}
```

Minimize:

```rust
if (wparam & 0xFFF0) == SC_MINIMIZE {
    hide_to_tray(hwnd);
    return 0;
}
```

---

## Theme switching

```rust
unsafe fn apply_theme(hwnd: HWND) {
    DeleteObject(G_BRUSH_BG);
    if G_DARK {
        G_BG_COLOR = CLR_DARK_BG; // 0x00000000
        G_TEXT_COLOR = CLR_DARK_TEXT;
        G_BRUSH_BG = CreateSolidBrush(CLR_DARK_BG);
    } else {
        G_BG_COLOR = CLR_LIGHT_BG;
        G_TEXT_COLOR = CLR_LIGHT_TEXT;
        G_BRUSH_BG = CreateSolidBrush(CLR_LIGHT_BG);
    }
    InvalidateRect(hwnd, null(), 1);
}
```

COLORREF is **0x00BBGGRR** (Windows BGR).

---

## Error string examples

```text
Administrator privileges required.
8/8 operations OK | Freed ~1024 MB | Standby purged | Threads done
Nothing selected.
```

UI maps -1 / 0 / >0 to Admin / Success / Partial message boxes.

---

## Security notes

- Needs full admin; not designed for restricted tokens.
- Does not send network data (DNS flush is local `ipconfig`).
- Does not install services or drivers.
- Tray residency is user-session only; logoff ends it.
- Temp delete only hits known temp roots, non-recursive.

---

## Extending with a new cleaner

1. Add `REDUCT_FOO 0x2000` in `engine.h`.
2. Implement `clean_foo_thread` in `engine.cpp`.
3. Fan-out branch in `sa_clean`.
4. Extend `sa_estimate_each` slot 13+ (grow array + UI).
5. New checkbox IDC + bit in `get_mask`.
6. Help string in `help_for`.
7. Bump README table.

Keep the worker short-lived and timeout-safe.

---

## Build.rs annotated

```rust
fn main() {
    let mut b = cc::Build::new();
    b.cpp(true);
    b.file("src/cpp/engine.cpp");
    b.include("src/cpp");
    b.define("UNICODE", None);
    b.define("_UNICODE", None);
    // static flags when supported
    b.compile("sacleaner_cpp");
    println!("cargo:rustc-link-lib=user32");
    // ... other libs
}
```

Rerun-if-changed on engine sources so edits rebuild the static lib.

---

## Cargo profile

```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
panic = "abort"
strip = true
```

Smaller binary, no unwind tables, harder to debug — fine for release utility.

---

## Testing checklist

- [ ] Non-admin launch → UAC relaunch path
- [ ] Admin launch → window shows live RAM
- [ ] Safe Profile clean → success box, last freed updates
- [ ] Uncheck all → Clean warns
- [ ] Double Clean click → “Already cleaning”
- [ ] Aggressive toggle label flips
- [ ] Theme dark/light + BG shade
- [ ] Threshold 90 → Start Auto → status shows 90%
- [ ] Stop Auto
- [ ] To Tray → icon visible → click restore
- [ ] Minimize → tray
- [ ] Maximize blocked
- [ ] Close → tray icon gone
- [ ] Temp enabled once on test VM

---

## Known limitations

- Estimates are heuristics.
- Nt info classes can differ across Windows builds.
- Shallow temp will not clear nested package caches.
- No per-process working-set picker (global policy only).
- Single-instance not enforced (two EXEs can race; critical section is per-process only).

---

## Operator recipes

### Before a game

```text
Safe Profile + Aggressive ON → CLEAN NOW → To Tray
```

### Low-RAM laptop overnight

```text
Threshold 70 → Start Auto → To Tray
```

### After large Visual Studio build

```text
Standby + Cache + Modified + Temp → CLEAN NOW
```

### Corporate machine (strict)

Stick to WS + Standby only; skip Temp/Prefetch.

---

## Source map by concern

| Concern | File | Symbols |
|---------|------|---------|
| Nt purge | engine.cpp | DoNtSetSysInfo, clean_*_thread |
| Estimates | engine.cpp | sa_estimate_each |
| Auto | engine.cpp | AutoCleanThread, sa_set_auto_threshold |
| Admin | engine.cpp | is_adm, sa_uac_restart |
| UI mask | main.rs | get_mask, set_check |
| Async | main.rs | WM_APP_CLEAN_DONE, G_CLEANING |
| Tray | main.rs | tray_add, hide_to_tray |
| Theme | main.rs | apply_theme, cycle_bg |
| Help | main.rs | help_for |

---

## Glossary (short)

- **Soft fault**: page still in RAM (standby); no disk.
- **Hard fault**: page must be read from disk/pagefile.
- **VACB**: Virtual Address Control Block (cache manager).
- **PFN**: Page Frame Number database entry.
- **Elevation**: admin token present.

---

## Appendix: full flag list

```c
REDUCT_WORKING_SET      0x0001
REDUCT_SYSTEM_CACHE     0x0002
REDUCT_STANDBY_LIST     0x0004
REDUCT_MODIFIED_LIST    0x0008
REDUCT_REGISTRY         0x0010
REDUCT_COMBINE          0x0020
REDUCT_LOW_PRIORITY     0x0040
REDUCT_FLUSH_VOLUMES    0x0080
REDUCT_CLIPBOARD        0x0100
REDUCT_DNS_CACHE        0x0200
REDUCT_TEMP_FILES       0x0400
REDUCT_PREFETCH         0x0800
REDUCT_THUMBNAIL_CACHE  0x1000
```

---

## Appendix: sample status line formats

```text
RAM  11.2 GB / 15.8 GB   (71% used)   Free: 4.6 GB
Standby: 3.1 GB   |   Modified: 240.0 MB   |   Cache: 1.8 GB
Processes: 312
Last freed: 889 MB
Total ~3.40 GB   [WS 316.0 MB | Cache 900.0 MB | Standby 2.20 GB]
Cleaning in background...
Ready
Auto-clean started — triggers at 75% RAM usage
```

---

## Appendix: thread list inside sa_clean

| Bit | Thread proc |
|-----|-------------|
| WS | clean_ws_thread |
| Cache | clean_cache_thread |
| Standby | clean_standby_thread |
| Modified | clean_modified_thread |
| Registry | clean_registry_thread |
| Combine | clean_combine_thread |
| LowPrio | clean_lowprio_thread |
| Clipboard | clean_clipboard_thread |
| DNS | clean_dns_thread |
| Temp | clean_temp_thread |
| Prefetch | clean_prefetch_thread |
| Thumb | clean_thumb_thread |
| Flush volumes | inline (no thread) |

---

## Appendix: files touched on disk (optional flags)

```text
%TEMP%\*
%LOCALAPPDATA%\Temp\*
%SystemRoot%\Temp\*
%SystemRoot%\Prefetch\*
%LOCALAPPDATA%\Microsoft\Windows\Explorer\thumbcache_*.db
```

Only when corresponding bits are set.

---

## Appendix: dependencies at runtime

- ntdll.dll (always)
- user32 gdi32 shell32 advapi32 kernel32 psapi
- ipconfig.exe only if DNS bit set

No .NET, no VC redist required if fully static GNU build succeeds.

---

## Appendix: versioning

```toml
version = "2.0.0"
```

Window title includes v2.0. Binary name suggestion: `sa_cleaner_v2.exe`.

---

## Appendix: license stance

Provided as-is for education and personal use. Authors are not liable for data loss from temp deletion or performance side effects of standby purges on servers.

---

## Appendix: contact with the codebase

When debugging:

1. Reproduce with Aggressive OFF and Safe mask.
2. Watch status line + Last freed.
3. If partial, note which Windows build (winver).
4. Confirm elevation with `whoami /groups` outside the tool.
5. For UI hangs, verify you are on a build with async clean (WM_APP_CLEAN_DONE present in main.rs).

---

## Appendix: future work ideas (not implemented)

- Named mutex single-instance
- JSON log file of free deltas
- Per-process WS trim list view
- Configurable poll interval in UI
- ARM64 Windows build
- Signed tray icon resource

---

## Appendix: related reading

- Windows Internals (memory management chapters)
- MSDN: `GlobalMemoryStatusEx`, `GetPerformanceInfo`
- ReactOS / phnt headers for system information class numbers
- Sysinternals RAMMap (to visualize standby before/after)

---


## FAQ

**Q1. Does this replace RAMMap?**

No. RAMMap visualizes; SA Cleaner acts. Use both.

Operational note for Q1: When validating this behavior, measure with Task Manager’s Performance tab and optionally RAMMap’s standby breakdown. Record free RAM before and after a Safe clean on an idle system, then again under memory pressure. Differences teach whether standby or working-set dominated the reclaim. Keep Aggressive off until baseline is understood.

**Q2. Will this raise FPS?**

Sometimes if the game was paging. Not guaranteed.

Operational note for Q2: When validating this behavior, measure with Task Manager’s Performance tab and optionally RAMMap’s standby breakdown. Record free RAM before and after a Safe clean on an idle system, then again under memory pressure. Differences teach whether standby or working-set dominated the reclaim. Keep Aggressive off until baseline is understood.

**Q3. Is standby purge harmful?**

It can increase hard faults temporarily after purge.

Operational note for Q3: When validating this behavior, measure with Task Manager’s Performance tab and optionally RAMMap’s standby breakdown. Record free RAM before and after a Safe clean on an idle system, then again under memory pressure. Differences teach whether standby or working-set dominated the reclaim. Keep Aggressive off until baseline is understood.

**Q4. Why not EmptyWorkingSet on each process?**

Global SysMemList is faster and simpler.

Operational note for Q4: When validating this behavior, measure with Task Manager’s Performance tab and optionally RAMMap’s standby breakdown. Record free RAM before and after a Safe clean on an idle system, then again under memory pressure. Differences teach whether standby or working-set dominated the reclaim. Keep Aggressive off until baseline is understood.

**Q5. Can I run without admin?**

No. Nt memory list ops need elevation.

Operational note for Q5: When validating this behavior, measure with Task Manager’s Performance tab and optionally RAMMap’s standby breakdown. Record free RAM before and after a Safe clean on an idle system, then again under memory pressure. Differences teach whether standby or working-set dominated the reclaim. Keep Aggressive off until baseline is understood.

**Q6. Does auto-clean use the same mask as the checkboxes?**

It uses REDUCT_ALL_SAFE, not the UI mask.

Operational note for Q6: When validating this behavior, measure with Task Manager’s Performance tab and optionally RAMMap’s standby breakdown. Record free RAM before and after a Safe clean on an idle system, then again under memory pressure. Differences teach whether standby or working-set dominated the reclaim. Keep Aggressive off until baseline is understood.

**Q7. Why shallow temp only?**

Deep recursion caused multi-minute freezes on large trees.

Operational note for Q7: When validating this behavior, measure with Task Manager’s Performance tab and optionally RAMMap’s standby breakdown. Record free RAM before and after a Safe clean on an idle system, then again under memory pressure. Differences teach whether standby or working-set dominated the reclaim. Keep Aggressive off until baseline is understood.

**Q8. Can I change poll interval in UI?**

Not in v2; edit engine sleep constants and rebuild.

Operational note for Q8: When validating this behavior, measure with Task Manager’s Performance tab and optionally RAMMap’s standby breakdown. Record free RAM before and after a Safe clean on an idle system, then again under memory pressure. Differences teach whether standby or working-set dominated the reclaim. Keep Aggressive off until baseline is understood.

**Q9. Does theme affect cleaning?**

No. Purely visual.

Operational note for Q9: When validating this behavior, measure with Task Manager’s Performance tab and optionally RAMMap’s standby breakdown. Record free RAM before and after a Safe clean on an idle system, then again under memory pressure. Differences teach whether standby or working-set dominated the reclaim. Keep Aggressive off until baseline is understood.

**Q10. What about Windows on ARM?**

Not validated; x64 is the target.

Operational note for Q10: When validating this behavior, measure with Task Manager’s Performance tab and optionally RAMMap’s standby breakdown. Record free RAM before and after a Safe clean on an idle system, then again under memory pressure. Differences teach whether standby or working-set dominated the reclaim. Keep Aggressive off until baseline is understood.

**Q11. Is there an installer?**

No. Single EXE.

Operational note for Q11: When validating this behavior, measure with Task Manager’s Performance tab and optionally RAMMap’s standby breakdown. Record free RAM before and after a Safe clean on an idle system, then again under memory pressure. Differences teach whether standby or working-set dominated the reclaim. Keep Aggressive off until baseline is understood.

**Q12. Does it survive reboot?**

No persistence beyond optional user shortcut.

Operational note for Q12: When validating this behavior, measure with Task Manager’s Performance tab and optionally RAMMap’s standby breakdown. Record free RAM before and after a Safe clean on an idle system, then again under memory pressure. Differences teach whether standby or working-set dominated the reclaim. Keep Aggressive off until baseline is understood.

**Q13. Why MessageBox on complete?**

Clear operator feedback; can be replaced with toast later.

Operational note for Q13: When validating this behavior, measure with Task Manager’s Performance tab and optionally RAMMap’s standby breakdown. Record free RAM before and after a Safe clean on an idle system, then again under memory pressure. Differences teach whether standby or working-set dominated the reclaim. Keep Aggressive off until baseline is understood.

**Q14. Can estimates exceed physical RAM?**

They are capped by measured components in practice but not hard-clamped.

Operational note for Q14: When validating this behavior, measure with Task Manager’s Performance tab and optionally RAMMap’s standby breakdown. Record free RAM before and after a Safe clean on an idle system, then again under memory pressure. Differences teach whether standby or working-set dominated the reclaim. Keep Aggressive off until baseline is understood.

**Q15. What if WaitForMultipleObjects times out?**

Handles closed; some work may still finish late.

Operational note for Q15: When validating this behavior, measure with Task Manager’s Performance tab and optionally RAMMap’s standby breakdown. Record free RAM before and after a Safe clean on an idle system, then again under memory pressure. Differences teach whether standby or working-set dominated the reclaim. Keep Aggressive off until baseline is understood.

**Q16. Is g_err thread-safe?**

Readers should only call after sa_clean returns / on UI thread after PostMessage.

Operational note for Q16: When validating this behavior, measure with Task Manager’s Performance tab and optionally RAMMap’s standby breakdown. Record free RAM before and after a Safe clean on an idle system, then again under memory pressure. Differences teach whether standby or working-set dominated the reclaim. Keep Aggressive off until baseline is understood.

**Q17. Why BELOW_NORMAL priority?**

Reduce contention with foreground games.

Operational note for Q17: When validating this behavior, measure with Task Manager’s Performance tab and optionally RAMMap’s standby breakdown. Record free RAM before and after a Safe clean on an idle system, then again under memory pressure. Differences teach whether standby or working-set dominated the reclaim. Keep Aggressive off until baseline is understood.

**Q18. Can I script it?**

Call the C API from your own host or add a CLI front-end.

Operational note for Q18: When validating this behavior, measure with Task Manager’s Performance tab and optionally RAMMap’s standby breakdown. Record free RAM before and after a Safe clean on an idle system, then again under memory pressure. Differences teach whether standby or working-set dominated the reclaim. Keep Aggressive off until baseline is understood.

**Q19. Does DNS flush need admin?**

Usually works elevated; without admin may fail silently.

Operational note for Q19: When validating this behavior, measure with Task Manager’s Performance tab and optionally RAMMap’s standby breakdown. Record free RAM before and after a Safe clean on an idle system, then again under memory pressure. Differences teach whether standby or working-set dominated the reclaim. Keep Aggressive off until baseline is understood.

**Q20. Thumbcache locked by Explorer?**

Delete may fail on locked files; ignored.

Operational note for Q20: When validating this behavior, measure with Task Manager’s Performance tab and optionally RAMMap’s standby breakdown. Record free RAM before and after a Safe clean on an idle system, then again under memory pressure. Differences teach whether standby or working-set dominated the reclaim. Keep Aggressive off until baseline is understood.


## API usage examples

### Example 1

```c
// Example 1: clean standby only
int rc = sa_clean(REDUCT_STANDBY_LIST);
```

Commentary 1: Pair this call with a user-visible status update. On failure, read `sa_get_err`. For automation, prefer explicit masks over ALL_SAFE when the host workload is known.

### Example 2

```c
// Example 2: query memory
unsigned long long t,u,f; sa_get_mem(&t,&u,&f);
```

Commentary 2: Pair this call with a user-visible status update. On failure, read `sa_get_err`. For automation, prefer explicit masks over ALL_SAFE when the host workload is known.

### Example 3

```rust
// Example 3: aggressive on
unsafe { sa_set_aggressive(1); }
```

Commentary 3: Pair this call with a user-visible status update. On failure, read `sa_get_err`. For automation, prefer explicit masks over ALL_SAFE when the host workload is known.

### Example 4

```c
// Example 4: auto at 80%
sa_set_auto_threshold(80);
```

Commentary 4: Pair this call with a user-visible status update. On failure, read `sa_get_err`. For automation, prefer explicit masks over ALL_SAFE when the host workload is known.

### Example 5

```c
// Example 5: clean standby only
int rc = sa_clean(REDUCT_STANDBY_LIST);
```

Commentary 5: Pair this call with a user-visible status update. On failure, read `sa_get_err`. For automation, prefer explicit masks over ALL_SAFE when the host workload is known.

### Example 6

```c
// Example 6: query memory
unsigned long long t,u,f; sa_get_mem(&t,&u,&f);
```

Commentary 6: Pair this call with a user-visible status update. On failure, read `sa_get_err`. For automation, prefer explicit masks over ALL_SAFE when the host workload is known.

### Example 7

```rust
// Example 7: aggressive on
unsafe { sa_set_aggressive(1); }
```

Commentary 7: Pair this call with a user-visible status update. On failure, read `sa_get_err`. For automation, prefer explicit masks over ALL_SAFE when the host workload is known.

### Example 8

```c
// Example 8: auto at 80%
sa_set_auto_threshold(80);
```

Commentary 8: Pair this call with a user-visible status update. On failure, read `sa_get_err`. For automation, prefer explicit masks over ALL_SAFE when the host workload is known.

### Example 9

```c
// Example 9: clean standby only
int rc = sa_clean(REDUCT_STANDBY_LIST);
```

Commentary 9: Pair this call with a user-visible status update. On failure, read `sa_get_err`. For automation, prefer explicit masks over ALL_SAFE when the host workload is known.

### Example 10

```c
// Example 10: query memory
unsigned long long t,u,f; sa_get_mem(&t,&u,&f);
```

Commentary 10: Pair this call with a user-visible status update. On failure, read `sa_get_err`. For automation, prefer explicit masks over ALL_SAFE when the host workload is known.

### Example 11

```rust
// Example 11: aggressive on
unsafe { sa_set_aggressive(1); }
```

Commentary 11: Pair this call with a user-visible status update. On failure, read `sa_get_err`. For automation, prefer explicit masks over ALL_SAFE when the host workload is known.

### Example 12

```c
// Example 12: auto at 80%
sa_set_auto_threshold(80);
```

Commentary 12: Pair this call with a user-visible status update. On failure, read `sa_get_err`. For automation, prefer explicit masks over ALL_SAFE when the host workload is known.

### Example 13

```c
// Example 13: clean standby only
int rc = sa_clean(REDUCT_STANDBY_LIST);
```

Commentary 13: Pair this call with a user-visible status update. On failure, read `sa_get_err`. For automation, prefer explicit masks over ALL_SAFE when the host workload is known.

### Example 14

```c
// Example 14: query memory
unsigned long long t,u,f; sa_get_mem(&t,&u,&f);
```

Commentary 14: Pair this call with a user-visible status update. On failure, read `sa_get_err`. For automation, prefer explicit masks over ALL_SAFE when the host workload is known.

### Example 15

```rust
// Example 15: aggressive on
unsafe { sa_set_aggressive(1); }
```

Commentary 15: Pair this call with a user-visible status update. On failure, read `sa_get_err`. For automation, prefer explicit masks over ALL_SAFE when the host workload is known.

### Example 16

```c
// Example 16: auto at 80%
sa_set_auto_threshold(80);
```

Commentary 16: Pair this call with a user-visible status update. On failure, read `sa_get_err`. For automation, prefer explicit masks over ALL_SAFE when the host workload is known.

### Example 17

```c
// Example 17: clean standby only
int rc = sa_clean(REDUCT_STANDBY_LIST);
```

Commentary 17: Pair this call with a user-visible status update. On failure, read `sa_get_err`. For automation, prefer explicit masks over ALL_SAFE when the host workload is known.

### Example 18

```c
// Example 18: query memory
unsigned long long t,u,f; sa_get_mem(&t,&u,&f);
```

Commentary 18: Pair this call with a user-visible status update. On failure, read `sa_get_err`. For automation, prefer explicit masks over ALL_SAFE when the host workload is known.

### Example 19

```rust
// Example 19: aggressive on
unsafe { sa_set_aggressive(1); }
```

Commentary 19: Pair this call with a user-visible status update. On failure, read `sa_get_err`. For automation, prefer explicit masks over ALL_SAFE when the host workload is known.

### Example 20

```c
// Example 20: auto at 80%
sa_set_auto_threshold(80);
```

Commentary 20: Pair this call with a user-visible status update. On failure, read `sa_get_err`. For automation, prefer explicit masks over ALL_SAFE when the host workload is known.

### Example 21

```c
// Example 21: clean standby only
int rc = sa_clean(REDUCT_STANDBY_LIST);
```

Commentary 21: Pair this call with a user-visible status update. On failure, read `sa_get_err`. For automation, prefer explicit masks over ALL_SAFE when the host workload is known.

### Example 22

```c
// Example 22: query memory
unsigned long long t,u,f; sa_get_mem(&t,&u,&f);
```

Commentary 22: Pair this call with a user-visible status update. On failure, read `sa_get_err`. For automation, prefer explicit masks over ALL_SAFE when the host workload is known.

### Example 23

```rust
// Example 23: aggressive on
unsafe { sa_set_aggressive(1); }
```

Commentary 23: Pair this call with a user-visible status update. On failure, read `sa_get_err`. For automation, prefer explicit masks over ALL_SAFE when the host workload is known.

### Example 24

```c
// Example 24: auto at 80%
sa_set_auto_threshold(80);
```

Commentary 24: Pair this call with a user-visible status update. On failure, read `sa_get_err`. For automation, prefer explicit masks over ALL_SAFE when the host workload is known.

### Example 25

```c
// Example 25: clean standby only
int rc = sa_clean(REDUCT_STANDBY_LIST);
```

Commentary 25: Pair this call with a user-visible status update. On failure, read `sa_get_err`. For automation, prefer explicit masks over ALL_SAFE when the host workload is known.

### Example 26

```c
// Example 26: query memory
unsigned long long t,u,f; sa_get_mem(&t,&u,&f);
```

Commentary 26: Pair this call with a user-visible status update. On failure, read `sa_get_err`. For automation, prefer explicit masks over ALL_SAFE when the host workload is known.

### Example 27

```rust
// Example 27: aggressive on
unsafe { sa_set_aggressive(1); }
```

Commentary 27: Pair this call with a user-visible status update. On failure, read `sa_get_err`. For automation, prefer explicit masks over ALL_SAFE when the host workload is known.

### Example 28

```c
// Example 28: auto at 80%
sa_set_auto_threshold(80);
```

Commentary 28: Pair this call with a user-visible status update. On failure, read `sa_get_err`. For automation, prefer explicit masks over ALL_SAFE when the host workload is known.

### Example 29

```c
// Example 29: clean standby only
int rc = sa_clean(REDUCT_STANDBY_LIST);
```

Commentary 29: Pair this call with a user-visible status update. On failure, read `sa_get_err`. For automation, prefer explicit masks over ALL_SAFE when the host workload is known.

### Example 30

```c
// Example 30: query memory
unsigned long long t,u,f; sa_get_mem(&t,&u,&f);
```

Commentary 30: Pair this call with a user-visible status update. On failure, read `sa_get_err`. For automation, prefer explicit masks over ALL_SAFE when the host workload is known.

### Example 31

```rust
// Example 31: aggressive on
unsafe { sa_set_aggressive(1); }
```

Commentary 31: Pair this call with a user-visible status update. On failure, read `sa_get_err`. For automation, prefer explicit masks over ALL_SAFE when the host workload is known.

### Example 32

```c
// Example 32: auto at 80%
sa_set_auto_threshold(80);
```

Commentary 32: Pair this call with a user-visible status update. On failure, read `sa_get_err`. For automation, prefer explicit masks over ALL_SAFE when the host workload is known.

### Example 33

```c
// Example 33: clean standby only
int rc = sa_clean(REDUCT_STANDBY_LIST);
```

Commentary 33: Pair this call with a user-visible status update. On failure, read `sa_get_err`. For automation, prefer explicit masks over ALL_SAFE when the host workload is known.

### Example 34

```c
// Example 34: query memory
unsigned long long t,u,f; sa_get_mem(&t,&u,&f);
```

Commentary 34: Pair this call with a user-visible status update. On failure, read `sa_get_err`. For automation, prefer explicit masks over ALL_SAFE when the host workload is known.

### Example 35

```rust
// Example 35: aggressive on
unsafe { sa_set_aggressive(1); }
```

Commentary 35: Pair this call with a user-visible status update. On failure, read `sa_get_err`. For automation, prefer explicit masks over ALL_SAFE when the host workload is known.

### Example 36

```c
// Example 36: auto at 80%
sa_set_auto_threshold(80);
```

Commentary 36: Pair this call with a user-visible status update. On failure, read `sa_get_err`. For automation, prefer explicit masks over ALL_SAFE when the host workload is known.

### Example 37

```c
// Example 37: clean standby only
int rc = sa_clean(REDUCT_STANDBY_LIST);
```

Commentary 37: Pair this call with a user-visible status update. On failure, read `sa_get_err`. For automation, prefer explicit masks over ALL_SAFE when the host workload is known.

### Example 38

```c
// Example 38: query memory
unsigned long long t,u,f; sa_get_mem(&t,&u,&f);
```

Commentary 38: Pair this call with a user-visible status update. On failure, read `sa_get_err`. For automation, prefer explicit masks over ALL_SAFE when the host workload is known.

### Example 39

```rust
// Example 39: aggressive on
unsafe { sa_set_aggressive(1); }
```

Commentary 39: Pair this call with a user-visible status update. On failure, read `sa_get_err`. For automation, prefer explicit masks over ALL_SAFE when the host workload is known.

### Example 40

```c
// Example 40: auto at 80%
sa_set_auto_threshold(80);
```

Commentary 40: Pair this call with a user-visible status update. On failure, read `sa_get_err`. For automation, prefer explicit masks over ALL_SAFE when the host workload is known.

### Example 41

```c
// Example 41: clean standby only
int rc = sa_clean(REDUCT_STANDBY_LIST);
```

Commentary 41: Pair this call with a user-visible status update. On failure, read `sa_get_err`. For automation, prefer explicit masks over ALL_SAFE when the host workload is known.

### Example 42

```c
// Example 42: query memory
unsigned long long t,u,f; sa_get_mem(&t,&u,&f);
```

Commentary 42: Pair this call with a user-visible status update. On failure, read `sa_get_err`. For automation, prefer explicit masks over ALL_SAFE when the host workload is known.

### Example 43

```rust
// Example 43: aggressive on
unsafe { sa_set_aggressive(1); }
```

Commentary 43: Pair this call with a user-visible status update. On failure, read `sa_get_err`. For automation, prefer explicit masks over ALL_SAFE when the host workload is known.

### Example 44

```c
// Example 44: auto at 80%
sa_set_auto_threshold(80);
```

Commentary 44: Pair this call with a user-visible status update. On failure, read `sa_get_err`. For automation, prefer explicit masks over ALL_SAFE when the host workload is known.

### Example 45

```c
// Example 45: clean standby only
int rc = sa_clean(REDUCT_STANDBY_LIST);
```

Commentary 45: Pair this call with a user-visible status update. On failure, read `sa_get_err`. For automation, prefer explicit masks over ALL_SAFE when the host workload is known.

### Example 46

```c
// Example 46: query memory
unsigned long long t,u,f; sa_get_mem(&t,&u,&f);
```

Commentary 46: Pair this call with a user-visible status update. On failure, read `sa_get_err`. For automation, prefer explicit masks over ALL_SAFE when the host workload is known.

### Example 47

```rust
// Example 47: aggressive on
unsafe { sa_set_aggressive(1); }
```

Commentary 47: Pair this call with a user-visible status update. On failure, read `sa_get_err`. For automation, prefer explicit masks over ALL_SAFE when the host workload is known.

### Example 48

```c
// Example 48: auto at 80%
sa_set_auto_threshold(80);
```

Commentary 48: Pair this call with a user-visible status update. On failure, read `sa_get_err`. For automation, prefer explicit masks over ALL_SAFE when the host workload is known.

### Example 49

```c
// Example 49: clean standby only
int rc = sa_clean(REDUCT_STANDBY_LIST);
```

Commentary 49: Pair this call with a user-visible status update. On failure, read `sa_get_err`. For automation, prefer explicit masks over ALL_SAFE when the host workload is known.

### Example 50

```c
// Example 50: query memory
unsigned long long t,u,f; sa_get_mem(&t,&u,&f);
```

Commentary 50: Pair this call with a user-visible status update. On failure, read `sa_get_err`. For automation, prefer explicit masks over ALL_SAFE when the host workload is known.

### Example 51

```rust
// Example 51: aggressive on
unsafe { sa_set_aggressive(1); }
```

Commentary 51: Pair this call with a user-visible status update. On failure, read `sa_get_err`. For automation, prefer explicit masks over ALL_SAFE when the host workload is known.

### Example 52

```c
// Example 52: auto at 80%
sa_set_auto_threshold(80);
```

Commentary 52: Pair this call with a user-visible status update. On failure, read `sa_get_err`. For automation, prefer explicit masks over ALL_SAFE when the host workload is known.

### Example 53

```c
// Example 53: clean standby only
int rc = sa_clean(REDUCT_STANDBY_LIST);
```

Commentary 53: Pair this call with a user-visible status update. On failure, read `sa_get_err`. For automation, prefer explicit masks over ALL_SAFE when the host workload is known.

### Example 54

```c
// Example 54: query memory
unsigned long long t,u,f; sa_get_mem(&t,&u,&f);
```

Commentary 54: Pair this call with a user-visible status update. On failure, read `sa_get_err`. For automation, prefer explicit masks over ALL_SAFE when the host workload is known.

### Example 55

```rust
// Example 55: aggressive on
unsafe { sa_set_aggressive(1); }
```

Commentary 55: Pair this call with a user-visible status update. On failure, read `sa_get_err`. For automation, prefer explicit masks over ALL_SAFE when the host workload is known.

### Example 56

```c
// Example 56: auto at 80%
sa_set_auto_threshold(80);
```

Commentary 56: Pair this call with a user-visible status update. On failure, read `sa_get_err`. For automation, prefer explicit masks over ALL_SAFE when the host workload is known.

### Example 57

```c
// Example 57: clean standby only
int rc = sa_clean(REDUCT_STANDBY_LIST);
```

Commentary 57: Pair this call with a user-visible status update. On failure, read `sa_get_err`. For automation, prefer explicit masks over ALL_SAFE when the host workload is known.

### Example 58

```c
// Example 58: query memory
unsigned long long t,u,f; sa_get_mem(&t,&u,&f);
```

Commentary 58: Pair this call with a user-visible status update. On failure, read `sa_get_err`. For automation, prefer explicit masks over ALL_SAFE when the host workload is known.

### Example 59

```rust
// Example 59: aggressive on
unsafe { sa_set_aggressive(1); }
```

Commentary 59: Pair this call with a user-visible status update. On failure, read `sa_get_err`. For automation, prefer explicit masks over ALL_SAFE when the host workload is known.

### Example 60

```c
// Example 60: auto at 80%
sa_set_auto_threshold(80);
```

Commentary 60: Pair this call with a user-visible status update. On failure, read `sa_get_err`. For automation, prefer explicit masks over ALL_SAFE when the host workload is known.

### Example 61

```c
// Example 61: clean standby only
int rc = sa_clean(REDUCT_STANDBY_LIST);
```

Commentary 61: Pair this call with a user-visible status update. On failure, read `sa_get_err`. For automation, prefer explicit masks over ALL_SAFE when the host workload is known.

### Example 62

```c
// Example 62: query memory
unsigned long long t,u,f; sa_get_mem(&t,&u,&f);
```

Commentary 62: Pair this call with a user-visible status update. On failure, read `sa_get_err`. For automation, prefer explicit masks over ALL_SAFE when the host workload is known.

### Example 63

```rust
// Example 63: aggressive on
unsafe { sa_set_aggressive(1); }
```

Commentary 63: Pair this call with a user-visible status update. On failure, read `sa_get_err`. For automation, prefer explicit masks over ALL_SAFE when the host workload is known.

### Example 64

```c
// Example 64: auto at 80%
sa_set_auto_threshold(80);
```

Commentary 64: Pair this call with a user-visible status update. On failure, read `sa_get_err`. For automation, prefer explicit masks over ALL_SAFE when the host workload is known.

### Example 65

```c
// Example 65: clean standby only
int rc = sa_clean(REDUCT_STANDBY_LIST);
```

Commentary 65: Pair this call with a user-visible status update. On failure, read `sa_get_err`. For automation, prefer explicit masks over ALL_SAFE when the host workload is known.

### Example 66

```c
// Example 66: query memory
unsigned long long t,u,f; sa_get_mem(&t,&u,&f);
```

Commentary 66: Pair this call with a user-visible status update. On failure, read `sa_get_err`. For automation, prefer explicit masks over ALL_SAFE when the host workload is known.

### Example 67

```rust
// Example 67: aggressive on
unsafe { sa_set_aggressive(1); }
```

Commentary 67: Pair this call with a user-visible status update. On failure, read `sa_get_err`. For automation, prefer explicit masks over ALL_SAFE when the host workload is known.

### Example 68

```c
// Example 68: auto at 80%
sa_set_auto_threshold(80);
```

Commentary 68: Pair this call with a user-visible status update. On failure, read `sa_get_err`. For automation, prefer explicit masks over ALL_SAFE when the host workload is known.

### Example 69

```c
// Example 69: clean standby only
int rc = sa_clean(REDUCT_STANDBY_LIST);
```

Commentary 69: Pair this call with a user-visible status update. On failure, read `sa_get_err`. For automation, prefer explicit masks over ALL_SAFE when the host workload is known.

### Example 70

```c
// Example 70: query memory
unsigned long long t,u,f; sa_get_mem(&t,&u,&f);
```

Commentary 70: Pair this call with a user-visible status update. On failure, read `sa_get_err`. For automation, prefer explicit masks over ALL_SAFE when the host workload is known.

### Example 71

```rust
// Example 71: aggressive on
unsafe { sa_set_aggressive(1); }
```

Commentary 71: Pair this call with a user-visible status update. On failure, read `sa_get_err`. For automation, prefer explicit masks over ALL_SAFE when the host workload is known.

### Example 72

```c
// Example 72: auto at 80%
sa_set_auto_threshold(80);
```

Commentary 72: Pair this call with a user-visible status update. On failure, read `sa_get_err`. For automation, prefer explicit masks over ALL_SAFE when the host workload is known.

### Example 73

```c
// Example 73: clean standby only
int rc = sa_clean(REDUCT_STANDBY_LIST);
```

Commentary 73: Pair this call with a user-visible status update. On failure, read `sa_get_err`. For automation, prefer explicit masks over ALL_SAFE when the host workload is known.

### Example 74

```c
// Example 74: query memory
unsigned long long t,u,f; sa_get_mem(&t,&u,&f);
```

Commentary 74: Pair this call with a user-visible status update. On failure, read `sa_get_err`. For automation, prefer explicit masks over ALL_SAFE when the host workload is known.

### Example 75

```rust
// Example 75: aggressive on
unsafe { sa_set_aggressive(1); }
```

Commentary 75: Pair this call with a user-visible status update. On failure, read `sa_get_err`. For automation, prefer explicit masks over ALL_SAFE when the host workload is known.

### Example 76

```c
// Example 76: auto at 80%
sa_set_auto_threshold(80);
```

Commentary 76: Pair this call with a user-visible status update. On failure, read `sa_get_err`. For automation, prefer explicit masks over ALL_SAFE when the host workload is known.

### Example 77

```c
// Example 77: clean standby only
int rc = sa_clean(REDUCT_STANDBY_LIST);
```

Commentary 77: Pair this call with a user-visible status update. On failure, read `sa_get_err`. For automation, prefer explicit masks over ALL_SAFE when the host workload is known.

### Example 78

```c
// Example 78: query memory
unsigned long long t,u,f; sa_get_mem(&t,&u,&f);
```

Commentary 78: Pair this call with a user-visible status update. On failure, read `sa_get_err`. For automation, prefer explicit masks over ALL_SAFE when the host workload is known.

### Example 79

```rust
// Example 79: aggressive on
unsafe { sa_set_aggressive(1); }
```

Commentary 79: Pair this call with a user-visible status update. On failure, read `sa_get_err`. For automation, prefer explicit masks over ALL_SAFE when the host workload is known.

### Example 80

```c
// Example 80: auto at 80%
sa_set_auto_threshold(80);
```

Commentary 80: Pair this call with a user-visible status update. On failure, read `sa_get_err`. For automation, prefer explicit masks over ALL_SAFE when the host workload is known.


## Per-flag operator reference

### Working Set

- Purpose: Trim process resident sets
- Risk: Safe
- Typical impact: Low
- When to enable: Enable **Working Set** when you understand the tradeoff (Safe). Disable it on hosts where trim process resident sets would disturb latency-sensitive workloads.
- Measurement tip: Compare standby and free counters before/after a clean with only **Working Set** selected to learn its marginal contribution on your hardware.

### System Cache

- Purpose: Shrink file cache
- Risk: Safe
- Typical impact: Medium
- When to enable: Enable **System Cache** when you understand the tradeoff (Safe). Disable it on hosts where shrink file cache would disturb latency-sensitive workloads.
- Measurement tip: Compare standby and free counters before/after a clean with only **System Cache** selected to learn its marginal contribution on your hardware.

### Standby List

- Purpose: Drop soft-fault cache pages
- Risk: Safe
- Typical impact: High
- When to enable: Enable **Standby List** when you understand the tradeoff (Safe). Disable it on hosts where drop soft-fault cache pages would disturb latency-sensitive workloads.
- Measurement tip: Compare standby and free counters before/after a clean with only **Standby List** selected to learn its marginal contribution on your hardware.

### Modified List

- Purpose: Write dirty pages then free
- Risk: Moderate disk
- Typical impact: Medium
- When to enable: Enable **Modified List** when you understand the tradeoff (Moderate disk). Disable it on hosts where write dirty pages then free would disturb latency-sensitive workloads.
- Measurement tip: Compare standby and free counters before/after a clean with only **Modified List** selected to learn its marginal contribution on your hardware.

### Registry

- Purpose: Reduce hive memory
- Risk: Safe
- Typical impact: Low
- When to enable: Enable **Registry** when you understand the tradeoff (Safe). Disable it on hosts where reduce hive memory would disturb latency-sensitive workloads.
- Measurement tip: Compare standby and free counters before/after a clean with only **Registry** selected to learn its marginal contribution on your hardware.

### Combine

- Purpose: Deduplicate identical pages
- Risk: Safe
- Typical impact: Low
- When to enable: Enable **Combine** when you understand the tradeoff (Safe). Disable it on hosts where deduplicate identical pages would disturb latency-sensitive workloads.
- Measurement tip: Compare standby and free counters before/after a clean with only **Combine** selected to learn its marginal contribution on your hardware.

### Low Priority

- Purpose: Cold standby only
- Risk: Safest standby
- Typical impact: Medium
- When to enable: Enable **Low Priority** when you understand the tradeoff (Safest standby). Disable it on hosts where cold standby only would disturb latency-sensitive workloads.
- Measurement tip: Compare standby and free counters before/after a clean with only **Low Priority** selected to learn its marginal contribution on your hardware.

### Flush Volumes

- Purpose: Force filesystem buffers to disk
- Risk: Disk sync
- Typical impact: Low
- When to enable: Enable **Flush Volumes** when you understand the tradeoff (Disk sync). Disable it on hosts where force filesystem buffers to disk would disturb latency-sensitive workloads.
- Measurement tip: Compare standby and free counters before/after a clean with only **Flush Volumes** selected to learn its marginal contribution on your hardware.

### Clipboard

- Purpose: Clear clipboard data
- Risk: Safe
- Typical impact: Trivial
- When to enable: Enable **Clipboard** when you understand the tradeoff (Safe). Disable it on hosts where clear clipboard data would disturb latency-sensitive workloads.
- Measurement tip: Compare standby and free counters before/after a clean with only **Clipboard** selected to learn its marginal contribution on your hardware.

### DNS Cache

- Purpose: Flush resolver cache
- Risk: Safe
- Typical impact: Trivial
- When to enable: Enable **DNS Cache** when you understand the tradeoff (Safe). Disable it on hosts where flush resolver cache would disturb latency-sensitive workloads.
- Measurement tip: Compare standby and free counters before/after a clean with only **DNS Cache** selected to learn its marginal contribution on your hardware.

### Temp Files

- Purpose: Delete top-level temp files
- Risk: Data loss in temp
- Typical impact: Variable
- When to enable: Enable **Temp Files** when you understand the tradeoff (Data loss in temp). Disable it on hosts where delete top-level temp files would disturb latency-sensitive workloads.
- Measurement tip: Compare standby and free counters before/after a clean with only **Temp Files** selected to learn its marginal contribution on your hardware.

### Prefetch

- Purpose: Clear prefetch traces
- Risk: Safe
- Typical impact: Low
- When to enable: Enable **Prefetch** when you understand the tradeoff (Safe). Disable it on hosts where clear prefetch traces would disturb latency-sensitive workloads.
- Measurement tip: Compare standby and free counters before/after a clean with only **Prefetch** selected to learn its marginal contribution on your hardware.

### Thumbnail Cache

- Purpose: Delete Explorer thumb DBs
- Risk: Safe
- Typical impact: Low
- When to enable: Enable **Thumbnail Cache** when you understand the tradeoff (Safe). Disable it on hosts where delete explorer thumb dbs would disturb latency-sensitive workloads.
- Measurement tip: Compare standby and free counters before/after a clean with only **Thumbnail Cache** selected to learn its marginal contribution on your hardware.

