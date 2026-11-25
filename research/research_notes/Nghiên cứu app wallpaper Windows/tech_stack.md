# Desktop tech stack for a lightweight Windows 10/11 wallpaper app (as of 25 Nov 2026)

Evidence labels used below:
- **[M]** measured, with a stated method. **[M-self]** measured by one project on one machine, self-reported in a PR/issue (several are from small repos, some visibly AI-assisted); not independently reproduced.
- **[Doc]** official documentation. **[Claim]** vendor or blog claim without a usable method.
- Memory metrics are not interchangeable: *working set* (WS) summed over processes double-counts shared pages; *private working set* / *USS* is roughly Task Manager's "Memory" column; *private bytes / private commit* also includes paged-out memory. Never compare across metrics.

## 1. Candidates and current stable versions (late Nov 2026)

### Takeaway
Tauri 2.12.x is the stable line (v3 is only alpha). All candidates are actively released except iced and gpui, which have had no crates.io release for about a year. .NET 8 and 9 both went out of support on 10 Nov 2026, so only .NET 10 or 11 is a sane .NET base.

### Cited Findings
- Tauri core 2.12.7 (18 Nov 2026, MSRV 1.90); 3.0.0-alpha.6 exists — [crates.io tauri](https://crates.io/api/v1/crates/tauri). CLI 2.12.6 (18 Nov 2026), JS API 2.12.5 — [npm CLI](https://registry.npmjs.org/@tauri-apps%2Fcli). wry 0.57.4 (25 Nov 2026) — [crates.io wry](https://crates.io/api/v1/crates/wry). tao 0.38.0 — [crates.io tao](https://crates.io/api/v1/crates/tao).
- Tauri 3 alphas: no official roadmap found; alpha.0 notes show runtime decoupling (Wry and CEF items removed from the root crate) — [newreleases.io alpha.0](https://newreleases.io/project/github/tauri-apps/tauri/release/tauri-v3.0.0-alpha.0) (read from a search extract, not fetched in full).
- Electron 45.5.0 (24 Nov 2026); 44.x and 43.x also maintained — [electron releases](https://github.com/electron/electron/releases). Windows 10 and up — [Electron README](https://github.com/electron/electron/blob/main/README.md).
- Windows App SDK 2.7.0 (17 Nov 2026), 1.8.12 (24 Sep 2026) — [WindowsAppSDK releases](https://github.com/microsoft/WindowsAppSDK/releases).
- .NET 11.0.0 (GA 10 Nov 2026) and .NET 10.0.15 (LTS), both listed with EOL 14 Nov 2028; .NET 8 and 9 EOL 10 Nov 2026 — [.NET 11 metadata](https://builds.dotnet.microsoft.com/dotnet/release-metadata/11.0/releases.json), [.NET 10](https://builds.dotnet.microsoft.com/dotnet/release-metadata/10.0/releases.json), [.NET 8](https://builds.dotnet.microsoft.com/dotnet/release-metadata/8.0/releases.json), [.NET 9](https://builds.dotnet.microsoft.com/dotnet/release-metadata/9.0/releases.json).
- Flutter 3.50.1 stable (20 Nov 2026) — [Flutter release index](https://storage.googleapis.com/flutter_infra_release/releases/releases_windows.json).
- Wails v3.1.0 (1 Nov 2026, not marked pre-release) and v2.17.1 (1 Nov 2026) — [Wails releases](https://github.com/wailsapp/wails/releases). (Some Wails doc mirrors still say "beta"; the GitHub release flag wins.)
- Slint 1.19.1 (24 Nov 2026) — [Slint releases](https://github.com/slint-ui/slint/releases). Avalonia 12.2.0 (23 Nov 2026) — [Avalonia releases](https://github.com/AvaloniaUI/Avalonia/releases).
- egui 0.37.0 (15 Oct 2026) — [crates.io](https://crates.io/api/v1/crates/egui); iced 0.14.0 (7 Dec 2025) — [crates.io](https://crates.io/api/v1/crates/iced); gpui 0.2.2 (22 Oct 2025) — [crates.io](https://crates.io/api/v1/crates/gpui); Dioxus 0.7.10 (30 Jul 2026, 0.8 alpha) — [crates.io](https://crates.io/api/v1/crates/dioxus); Freya 0.5.4 — [crates.io](https://crates.io/api/v1/crates/freya).

### Inferences
- Pin to Tauri 2.12.x; do not start on v3 alpha. egui/iced/gpui/Dioxus are outside the shortlist: no Windows measurements found for this use case, and iced/gpui release cadence on crates.io is slow.

### Gaps
- No reliable Windows memory/size/startup figures found for iced, gpui, Dioxus or Freya.

## 2. Installer size, idle RAM, cold start, with method

### Takeaway
Tauri wins installer size by about 100x (1 MB vs 106 MB for Electron), but on Windows with a window open its memory is essentially the same as Electron's once WebView2 children are counted (142 vs 149 MB private). WPF uses about half of that (74 MB) with a window, but needs a runtime or a ~56-60 MB self-contained payload. Cold start is NOT a proven Tauri win: the one multi-framework CI benchmark shows Electron starting faster than Tauri.

### Cited Findings
**Best like-for-like test: Dawloom, Oct 2026 [M].** Same counter app; i7-1185G7, 32 GB, Win 11 build 26200; private memory summed over the whole process tree *including WebView2 helpers*, 15 s after launch. Measured 13-14 Oct 2026 — [Dawloom benchmark](https://dawloom.com/blog/tauri-vs-electron-vs-wpf-benchmark/). (Author is an agency selling both Tauri and Electron work; startup was timed but not published because CPU was at 100%.)

| Build | Installer | Zipped | On disk | Private mem (tree) | WS (tree) | Processes | Runs |
|---|---|---|---|---|---|---|---|
| Tauri 2.12.3 | 1.05 MB NSIS | 1.21 MB | 3.0 MB | 142 MB | 333 MB | 7 | 5 (median) |
| Electron 44.4.5 | 106.1 MB NSIS | 150.8 MB | 367.4 MB | 149 MB | 298.6 MB | 4 | 1 |
| Wails 2.16 | not built | 4.50 MB | 11.2 MB | 205-209 MB | 361-365 MB | 7 | 2 |
| WPF .NET 10, framework-dependent | not built | 0.07 MB | 0.16 MB | 74.3 MB | 122.6 MB | 1 | 5 (median) |
| WPF .NET 10, self-contained | not built | 56-60 MB | 62-140 MB | not measured | - | - | - |
| Avalonia 12.1 Native AOT | not built | 13.7 MB | 34.7 MB | not measured | - | - | - |

**Cross-check [M-self]:** a separate probe of a stock Tauri 2.12.4 app (Win 11 26H2, WebView2 156, whole process tree, 8 processes) gave 72-75 MB *private working set* and 141.8 MB *private commit* — [tauri#16618, 22 Nov 2026](https://github.com/tauri-apps/tauri/issues/16618). This agrees with Dawloom's 142 MB "private" and shows that the metric alone changes the headline by 2x.

**Multi-framework CI benchmark (Elanis), last update 11/2026 [M, high noise]** — empty app, Windows x64, Release, GitHub CI; "used memory for main process and children"; the repo warns of a margin of error — [web-to-desktop-framework-comparison](https://github.com/Elanis/web-to-desktop-framework-comparison):

| | Build size | Used (main+children) | Free-RAM delta | Start |
|---|---|---|---|---|
| Electron | ~373 MB | ~279 MB | ~110 MB | ~195 ms |
| Tauri | ~3 MB | ~312 MB | ~187 MB | ~650 ms |
| Wails | ~11 MB | ~330 MB | ~193 MB | ~572 ms |
| Flutter | ~25 MB | ~377 MB | ~86 MB | ~2008 ms |

**Tauri cold start on Windows [M-self]:** CI (GPU-less), 5 runs, 3 Nov 2026: native splash 76 ms median, page first paint 524 ms median; app's own share of startup only 29 ms. On a real HP laptop the earlier version took ~1.3 s to first paint, of which ~1.1 s was WebView2 startup. Empty app 143 MB total, 17 MB private for the Rust process — [Radiologytutor PR 148](https://github.com/drskatyal/Radiologytutor/pull/148).

**WinUI 3 vs WPF tray shell [M-self], Oct 2026**, .NET 10, self-contained, ReadyToRun; medians of 5 — [benchbar PR 70](https://github.com/askysh/benchbar/pull/70):

| | Laptop WinUI | Laptop WPF | CI WinUI | CI WPF |
|---|---|---|---|---|
| Private bytes at rest in tray | 73.5 MB | 16.7 MB | 23.6 MB | 16.6 MB |
| WS at rest | 93.3 MB | 64.0 MB | 70.0 MB | 63.5 MB |
| Click to flyout, first | 222 ms | 302 ms | 162 ms | 131 ms |
| Click to flyout, warm | 12 ms | 5 ms | 16 ms | 11 ms |
| Self-contained size | 276 MB | 186.5 MB | | |
| Framework-dependent size | 94.8 MB | 25.5 MB | | |

- That project shipped WPF: WinUI's resting memory follows the GPU driver because the XAML compositor creates a D3D device even with no window — [same PR](https://github.com/askysh/benchbar/pull/70).
- WinUI 3 vs WPF, catalog rows (Windows App SDK 2.5.1, Oct 2026) [M-self]: self-contained 229 MiB vs 140 MiB; at 100k rows WinUI 200 MiB private vs WPF 400 MiB; WinUI scroll callback latency lower but with large variance; project still said no-go on WinUI because of TableView defects — [dj-library issue 4](https://github.com/lxsdd/dj-library-public/issues/4).
- Windows App SDK + NativeAOT [vendor Claim, 2024, SDK 1.6]: up to 50% faster start, ~2x smaller self-contained package in a sample — [Windows Report](https://windowsreport.com/microsoft-unveils-windows-app-sdk-1-6-with-native-aot-support-improved-controls-and-more/). Microsoft's May 2026 WinUI update is reported as cutting File Explorer allocations by 41% and WinUI launch code time by 25% — [Windows Latest, May 2026](https://www.windowslatest.com/2026/05/15/microsoft-commits-to-native-ui-for-windows-11-as-users-push-back-against-web-app-slop/) (search extract). A WinUI memory-growth fix (visual-state storyboards, theme resources) was only in SDK 2.5 *Experimental* on 2 Oct 2026 — [Windows Latest](https://www.windowslatest.com/2026/10/02/windows-11s-winui-just-got-basic-features-and-less-ram-hogging-after-years-of-losing-apps-to-the-web/).

**Slint on Windows [M-self]:**
- Tray app moved from Tauri to Slint 1.19 (software renderer, winit): 5.8 MB private in tray, 10.5 MB with settings window open, no msedgewebview2.exe (23 Nov 2026). Trade-offs: no hover tooltips, lost self-update — [claude-rpc PR 7](https://github.com/inerthel-agi/claude-rpc/pull/7). No Tauri baseline was published there.
- Slint RAM depends on renderer: same app ~19.5 MB WS on 1.17.1 Skia (which was defaulting to software) vs ~155.5 MB on 1.18.0 Skia (GPU via wgpu/DX12), ~46 MB with skia-opengl; closed "not planned" (GPU default kept on purpose) — [slint#13470, Sep 2026](https://github.com/slint-ui/slint/issues/13470).
- Slint renderers: software (no GPU acceleration), FemtoVG (wgpu/D3D), Skia ("heavy disk-footprint"), Vello experimental — [Slint docs](https://docs.slint.dev/latest/docs/slint/guide/backends-and-renderers/backends_and_renderers/).

**Flutter Windows (2024 data, 3.22):** counter sample 18-33 MB running, ~18 MB on disk — [flutter#148641](https://github.com/flutter/flutter/issues/148641). Conflicts with the 2026 CI figure above (377 MB used, 86 MB free-RAM delta), so treat Flutter RAM as unknown.

**The benchmark trap, documented:**
- Elanis and Dawloom count children; many blog claims (e.g. Tauri "42 MB idle") do not state what was counted, and 2026 posts contradict each other — example: [tech-insider 2026](https://tech-insider.org/tauri-vs-electron-2026/) (seen only through search extracts; another tech-insider page now returns 410 Gone). Do not cite them.
- The classic dispute: in 2022 a Postman engineer measured Tauri at 399 MB vs Electron 318 MB on Windows 10 for postman.com, 370 vs 312 MB for vscode.dev (Resource Monitor), and argued Tauri's own benchmark mishandled shared memory; a Tauri member replied that their benchmarks are a smoke/regression test, not a scientific report — [tauri#5889](https://github.com/tauri-apps/tauri/issues/5889) (closed 2024).
- The opposite trap: summing WS over-counts. One app read as "~540 MB" by summed WS is ~191 MB USS (WebView2 renderer 36 + GPU 33 + browser 31 + network 8 + storage 3 + crashpad 2 MB, Tauri shell 4 MB; the remaining 74 MB is that app's Python backend) — [Mellow PR 115, Nov 2026](https://github.com/jenox645/Mellow/pull/115). So WebView2 alone was about 113 MB there.

### Inferences
- With the window open: WPF (~74 MB) < Tauri ≈ Electron (~145 MB) < Wails (~205 MB) on private bytes. "Tauri uses 10x less RAM than Electron" is false on Windows.
- The user's "fast" criterion is the weakest part of the Tauri case: WebView2 creation dominates startup (~0.5 s on CI, ~1.1 s on one laptop). A native splash or starting the WebView early, and autostart launching with no window, hide this.
- WinUI 3 looks worst for this brief: largest payload (95-276 MB in a tray shell), and higher tray RAM than WPF on both machines.

### Gaps
- No single 2026 benchmark covers Tauri, Electron, WinUI 3, WPF, Avalonia, Flutter, Slint under one method. No published, trustworthy cold-start comparison exists (Dawloom withheld theirs).
- No independent WinUI 3 + NativeAOT on SDK 2.x memory/startup numbers found.
- No measured image-grid scroll performance for any of these stacks; must be tested.

## 3. Tauri memory in the tray: hide vs suspend vs destroy

### Takeaway
Hiding the window frees nothing. Destroying the window does: one Nov 2026 project measured 0 WebView2 processes / 0 MB after close-to-tray and on minimized start, leaving only the Rust process. That makes tray idle RAM competitive with native apps, at the cost of a reload when reopening, for which no measured time was found.

### Cited Findings
- `WebviewWindow::hide()` does not touch the WebView2 controller; tree private WS stays 72-75 MB whether shown, hidden or `SetIsVisible(false)` (the latter does drop CPU to 0%, e.g. 15.7% to 0% for an animated page) — [tauri#16618](https://github.com/tauri-apps/tauri/issues/16618).
- Same issue: `TrySuspend` drops the tree from ~72 MB to **6.3 MB private WS** but only ~25-30 s later, and private commit stays ~142 MB (pages are paged out, not freed); after `Resume` 29 MB; `Resume` takes 48-70 µs. Tauri 2.12.7 exposes neither `MemoryUsageTargetLevel` nor `TrySuspend`; apps need unsafe COM calls through `with_webview` and should pin a Tauri minor version. The feature request (opened 22 Nov 2026) is open — [tauri#16618](https://github.com/tauri-apps/tauri/issues/16618).
- A contrary data point: another project reports `TrySuspend` leaving ~458 MB (working set) of 504 MB across 8 processes, vs 47.6 MB for the Rust host; it lists "does the process tree exit when every window is closed?" as still to be tested (24 Oct 2026) — [rightkit#955](https://github.com/bogusyogi/rightkit/issues/955). Metric differs from the line above.
- `MemoryUsageTargetLevel=Low` while hidden: 189-192 to 170-174 MB USS, reopen still 0.06 s — [Mellow PR 115](https://github.com/jenox645/Mellow/pull/115). Small gain.
- Hidden window cost measured on another app: Rust process 29 MB WS / **6 MB private**; 7 WebView2 processes 440 MB WS / 278 MB private — [footnote#516, 22 Nov 2026](https://github.com/bobcat88/footnote/issues/516).
- **Destroy measured:** release build, private memory of WebView2 tree: window open 6 processes ~204 MB; closed to tray **0 processes, 0 MB**; reopened twice 6 processes ~207 MB (no growth); `--minimized` start 0 processes. Implementation: `"create": false` in tauri.conf.json, rebuild window from config on demand, `RunEvent::ExitRequested` without code calls `prevent_exit` — [BettrFTHR PR 8, 22 Nov 2026](https://github.com/H4ck3rPr0G4m3r/BettrFTHR/pull/8).
- Same pattern in footnote PR 719 (25 Nov 2026), which also persists the last route and adds reopen-latency instrumentation, but explicitly did not run the live measurement — [PR 719](https://github.com/bobcat88/footnote/pull/719). Mellow closes the hidden window for real after 10 minutes — [PR 115](https://github.com/jenox645/Mellow/pull/115).
- Tauri maintainer (2023, accepted answer): close the window, recreate from Rust on tray events, prevent exit on last window closed — [discussion 6308](https://github.com/tauri-apps/tauri/discussions/6308). `app.windows[].create=false` is documented — [Tauri config](https://v2.tauri.app/reference/config/).
- Known rough edges (from search extracts, not fetched in full): `prevent_exit` also blocks normal quit unless you distinguish exit codes — [tauri#13511](https://github.com/tauri-apps/tauri/issues/13511); loop with the window-state plugin on macOS — [discussion 11489](https://github.com/tauri-apps/tauri/discussions/11489); `AppHandle::exit(0)` can leave WebView2 children alive unless windows are destroyed first — [AstrBot-Go PR 177](https://github.com/mantoujun12/AstrBot-Go/pull/177).
- Microsoft: WebView2 cold launch must spin up processes and disk caches; avoid WebView2 for initial UI, keep the user data folder on a local fast disk, share one environment, use `MemoryUsageTargetLevel.Low` or `TrySuspendAsync` for inactive views — [WebView2 performance guide](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/performance). Its "reuse rather than re-create" advice is about speed when navigating between pages.
- A running app keeps using the old WebView2 version until it releases the environment or restarts — [WebView2 distribution](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/distribution).

### Inferences
- Expected tray footprint with destroy-on-hide: the Rust host only, roughly 5-20 MB private (data points: 4 MB USS, 6 MB private, 17 MB private, all measured while a window existed or was hidden). That is in the same band as WPF's 16.7 MB and near Slint's 5.8 MB. Not directly measured as one total after destroy.
- Good design: autostart with `--minimized` and no window; tray menu and wallpaper scheduler in Rust; destroy on close (optionally after a grace period of a few minutes for fast reopen); persist UI state in Rust/store; use a native tray menu for quick actions (next wallpaper, pause) so most interactions never start WebView2.
- Destroy-on-hide also lets WebView2 security updates take effect without restarting the app.

### Gaps
- Reopen-to-first-paint after destroy: no published number found. By analogy with the cold-start data (~0.5 s CI, ~1.1 s one laptop) it is plausibly 0.3-1 s; must be measured.
- No official Tauri guidance page for "destroy on hide" in v2 was found.

## 4. WebView2 runtime availability and Tauri installer options

### Takeaway
WebView2 is part of Windows 11 and on the vast majority of Windows 10 PCs; Tauri's default downloader adds 0 MB and covers the rest. Offline modes add ~127-180 MB and defeat the size goal.

### Cited Findings
- WebView2 is included in Windows 11. "The vast majority of Windows 10 devices" have it, a small number do not; apps should check and install — [Microsoft distribution doc](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/distribution).
- Edge and WebView2 Runtime keep updating on Windows 10 22H2 **until at least October 2028, without needing ESU** — [Edge supported OS, 9 Nov 2026](https://learn.microsoft.com/en-us/deployedge/microsoft-edge-supported-operating-systems).
- Tauri install modes ([Tauri Windows installer](https://v2.tauri.app/distribute/windows-installer/)):

| Mode | Internet needed | Added size |
|---|---|---|
| downloadBootstrapper (default) | yes | 0 MB |
| embedBootstrapper | yes | ~1.8 MB |
| offlineInstaller | no | ~127 MB |
| fixedVersion / fixedRuntime | no | ~180 MB |
| skip | - | 0 MB (not recommended) |

- Microsoft says Fixed Version binaries are over 250 MB (conflicts with Tauri's ~180 MB; likely uncompressed vs packed) and on Windows 10 unpackaged apps need extra `icacls` grants since v120 — [Microsoft](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/distribution).
- Gotcha: tauri-runtime-wry 2.12.3 checks for the runtime once at init and caches the answer; an app that installs WebView2 itself at first run must do so in `main` before building the Tauri runtime, or first launch fails with exit code 2 — [memry#3915, Nov 2026](https://github.com/memrynote/memry/issues/3915). Does not apply when the NSIS installer installs it.

### Inferences
- Use NSIS + `downloadBootstrapper` for direct downloads; keep a second config with `offlineInstaller` only if a Store EXE listing is wanted.

### Gaps
- No percentage figure for Windows 10 PCs lacking WebView2 was found.

## 5. Windows 10 status and framework support

### Takeaway
Windows 10 support ended 14 Oct 2025, but the free consumer ESU programme was extended to 12 Oct 2027 and Windows 10 is still ~25% of Windows desktops, so supporting it is worthwhile. Tauri/WebView2, Electron and Flutter officially support it; .NET 10/11 officially supports only LTSC/Enterprise editions of it. Mica is Windows 11 only.

### Cited Findings
- End of support 14 Oct 2025, 22H2 final version — [Microsoft Lifecycle](https://learn.microsoft.com/en-us/lifecycle/products/windows-10-home-and-pro).
- Consumer ESU extended on 25 Jun 2026 to **12 Oct 2027**, existing enrolments roll over — [BleepingComputer](https://bleepingcomputer.com/news/microsoft/microsoft-quietly-extends-free-windows-10-esu-support-to-october-2027), [Help Net Security](https://www.helpnetsecurity.com/2026/06/26/microsoft-windows-10-free-security-updates-esu-program/); requires 22H2, security updates only; commercial ESU runs to Oct 2028 (search extracts; Microsoft's own ESU page redirected and was not read).
- Share of Windows desktops, Oct 2026: Windows 11 74.29%, Windows 10 25.13% — [StatCounter](https://gs.statcounter.com/os-version-market-share/windows/desktop/worldwide).
- Windows App SDK 2.x lists Win10 1809, 21H2, 22H2 and all Win11; but each OS is supported only until its latest servicing end date, and the dated table lists only LTSC-type Windows 10 entries — [Windows App SDK support, 24 Sep 2026](https://learn.microsoft.com/en-us/windows/apps/windows-app-sdk/support).
- .NET 10 and 11: "Windows 10 support is limited to Long-Term Servicing Channel (LTSC) and Enterprise editions only" — [Install .NET on Windows, 11 Nov 2026](https://learn.microsoft.com/en-us/dotnet/core/install/windows).
- Tauri: Windows 10 and above — [Tauri README](https://github.com/tauri-apps/tauri/blob/dev/README.md). Rust Windows targets: Windows 10+ — [rustc platform support](https://doc.rust-lang.org/nightly/rustc/platform-support.html). Flutter 3.50: Windows 10 and 11 — [Flutter](https://docs.flutter.dev/reference/supported-platforms). Electron: Windows 10+ — [Electron](https://github.com/electron/electron/blob/main/README.md).
- Mica falls back to a solid colour when Windows is below build 22000 (also with transparency off, battery saver, low-end hardware, inactive window) — [Mica doc, Jul 2026](https://learn.microsoft.com/en-us/windows/apps/design/style/mica).
- Tauri `Effect`: Mica/Tabbed = Windows 11 only; Acrylic = Windows 10/11 but poor resize/drag performance on Win10 1903+ and Win11 22000; Blur = Win 7/10/11 (22H1) — [tauri Effect docs 2.12.7](https://docs.rs/tauri/2.12.7/tauri/window/enum.Effect.html).
- WPF on .NET 11 RC1 fixed Fluent backdrop issues on Windows 10 — [WPF what's new](https://learn.microsoft.com/en-us/dotnet/desktop/wpf/whats-new/net110). WPF `ThemeMode` (Fluent) was still experimental in .NET 10 — [benchbar PR 70](https://github.com/askysh/benchbar/pull/70).

### Inferences
- Windows 10 Home/Pro + .NET 10/11 will very likely run, but it is outside Microsoft's stated support; that counts against WPF/WinUI/Avalonia for a Win10+11 consumer app. WebView2's Oct 2028 commitment counts for Tauri.
- Design for a solid/gradient background on Win10 and enable Mica on build ≥ 22000; avoid acrylic on Win10.

### Gaps
- Electron/Chromium's planned end date for Windows 10 was not found.

## 6. Tauri v2 pieces needed

### Takeaway
Every building block exists as an official, recently released plugin or core API; only the WebView2 memory controls are missing. Setting the wallpaper itself still needs Rust Win32 code or a crate (not covered here).

### Cited Findings
- Official plugins, all listed for Windows: autostart, single-instance, store, http, updater, notification, window-state, deep-link, sql, global-shortcut, opener; all need Rust ≥ 1.90 — [plugins-workspace README](https://github.com/tauri-apps/plugins-workspace). Latest: autostart 2.7.0, single-instance 2.5.5, store 2.5.1, http 2.9.0, updater 2.14.1, notification 2.8.1 — crates.io: [autostart](https://crates.io/api/v1/crates/tauri-plugin-autostart), [single-instance](https://crates.io/api/v1/crates/tauri-plugin-single-instance), [store](https://crates.io/api/v1/crates/tauri-plugin-store), [http](https://crates.io/api/v1/crates/tauri-plugin-http), [updater](https://crates.io/api/v1/crates/tauri-plugin-updater), [notification](https://crates.io/api/v1/crates/tauri-plugin-notification).
- Autostart can pass arbitrary launch args (for a `--minimized` start) and has enable/disable/isEnabled, which must be allowed in capabilities — [autostart](https://v2.tauri.app/plugin/autostart/).
- Asset protocol: off by default, enabled with a path scope such as `$APPDATA/...` — [config](https://v2.tauri.app/reference/config/); add `asset:` and `http://asset.localhost` to `img-src` in the CSP — [CSP](https://v2.tauri.app/security/csp/).
- Custom protocol: `register_asynchronous_uri_scheme_protocol` answers off the main thread; on Windows the URL is `http://<scheme>.localhost/<path>` — [Builder docs](https://docs.rs/tauri/2.12.7/tauri/struct.Builder.html).
- Updater: signature verification cannot be disabled, static JSON hosting works (S3, GitHub), Windows modes passive (default)/basicUi/quiet, app exits automatically while installing on Windows; losing the key means no more updates — [updater](https://v2.tauri.app/plugin/updater/).
- NSIS default installs per-user with no admin rights, under %LOCALAPPDATA% — [Windows installer](https://v2.tauri.app/distribute/windows-installer/).

### Inferences
- Thumbnails: download and cache in Rust, serve through a custom async protocol (`thumb://`), with an on-disk cache and pre-resized WebP/JPEG; never send image bytes over IPC. Do provider calls in Rust (API keys and CORS stay out of the webview).

### Gaps
- Where autostart writes on Windows (HKCU Run) is not stated on the docs page.

## 7. Build toolchain on the developer's machine

### Takeaway
Tauri officially supports only the MSVC Windows target. The machine needs Microsoft C++ Build Tools ("Desktop development with C++", which brings the Windows SDK) and the default toolchain switched to the already-installed stable-msvc. Nothing else is missing.

### Cited Findings
- "Tauri officially only supports the MSVC Windows target" — [Windows installer docs](https://v2.tauri.app/distribute/windows-installer/). Prerequisites: install Microsoft C++ Build Tools with "Desktop development with C++", WebView2 (skip if present), and MSVC as default host triple: `rustup default stable-msvc` — [prerequisites](https://v2.tauri.app/start/prerequisites/). VBSCRIPT optional feature only for MSI builds — [same](https://v2.tauri.app/start/prerequisites/).
- GNU history: maintainers said they do not officially support or test it; apps failed with WebView2Loader.dll missing, fixed by copying the DLL in tauri-build, later fixes in tauri-bundler 2.2.2 and for nightly Cargo layouts — [PR/issue 4319](https://www.github.com/tauri-apps/tauri/pull/4319), [tauri-bundler 2.2.2](https://v2.tauri.app/release/tauri-bundler/v2.2.2/) (search extracts).
- On GNU, tauri-winres needs `windres` (MinGW64) on PATH to compile resources — [tauri-winres source](https://github.com/tauri-apps/winres/blob/HEAD/src/lib.rs).
- Rust itself treats both x86_64-pc-windows-msvc and -gnu as Tier 1 with host tools, -gnullvm Tier 2 — [rustc platform support](https://doc.rust-lang.org/nightly/rustc/platform-support.html).
- Read-only check of this machine (local observation, no URL): rustc 1.99.0; toolchains stable-gnu (default) and stable-msvc; no `Microsoft Visual Studio` folders, no `Windows Kits\10\bin`, no `cl.exe`, no gcc on PATH; `link.exe` found in Git Bash is the coreutils `link`, not MSVC's; WebView2 158.0.4370.33; `dotnet --list-sdks` empty.

### Inferences
- Steps: (1) install Build Tools 2022/2026 with the C++ workload (several GB of disk; needs admin); (2) in the project pin MSVC with a `rust-toolchain.toml`/`rustup override` instead of changing the global default if other projects rely on GNU; (3) build from PowerShell/cmd rather than Git Bash so that the coreutils `link.exe` cannot shadow MSVC's; (4) Node 24 and pnpm 12 are already fine.
- The current GNU default without MinGW/windres would most likely fail at the resource step. rustc 1.99 is above Tauri's MSRV (1.90) and Slint's (1.95). WPF/Avalonia/WinUI would need a .NET SDK install; Building Slint with the Skia renderer also needs Visual Studio/MSVC ([Slint docs](https://docs.slint.dev/latest/docs/slint/guide/backends-and-renderers/backends_and_renderers/)).

### Gaps
- Not test-built (task forbids building). Exact Build Tools disk size not sourced.

## 8. Frontend for Tauri and image-grid virtualisation

### Takeaway
Svelte 5 is the pragmatic pick: 9.7 KB compressed and near-vanilla speed, with 2 maintained virtualisers. Solid is smaller still (4.5 KB in v1.9), but Solid 2.0 is very new. React costs about 5x the bytes of Svelte and about 4x the first paint in this benchmark.

### Cited Findings
- js-framework-benchmark official run, Chrome 158 (2026), MacBook Pro M4, keyed; medians I computed from the published result data — [Chrome 158 results](https://krausest.github.io/js-framework-benchmark/2026/chrome158.html):

| Framework | Compressed KB | First paint ms | Create 1k ms | Create 10k ms | Swap rows ms | Run memory MB |
|---|---|---|---|---|---|---|
| vanilla JS | 2.5 | 49.5 | 20.2 | 205 | 10.9 | 1.86 |
| Solid 1.9.3 | 4.5 | 47.5 | 21.1 | 221 | 12.3 | 2.67 |
| Preact hooks 11.0 | 5.9 | 60.3 | 25.1 | 265 | 23.1 | 3.26 |
| Svelte 5.42.1 | 9.7 | 61.8 | 21.7 | 223 | 12.8 | 2.86 |
| Vue Vapor 3.6-rc.5 | 18.7 | 85.1 | 21.4 | 222 | 12.6 | 3.09 |
| Vue 3.5.39 | 23.3 | 94.8 | 23.7 | 242 | 13.0 | 3.91 |
| React 19.2 hooks | 51.4 | 228.9 | 24.6 | 342 | 87.4 | 4.39 |

- The benchmark itself says it covers specific workloads and does not establish a universally fastest framework — [llms.txt](https://krausest.github.io/js-framework-benchmark/llms.txt). Absolute values are for an M4 Mac, not Windows.
- Current versions: Svelte 5.59.1, Solid 2.0.4 (published 25 Nov 2026), React 19.3.1, Vue 3.5.46, Vite 8.3.8 — npm: [svelte](https://registry.npmjs.org/svelte), [solid-js](https://registry.npmjs.org/solid-js), [react](https://registry.npmjs.org/react), [vue](https://registry.npmjs.org/vue), [vite](https://registry.npmjs.org/vite).
- virtua 0.55.0: ~3 KB gzipped, Svelte ≥ 5, Solid ≥ 1.0, Vue, React; has VGrid and VMasonry; items that resize after async image load can jump — [virtua README](https://github.com/inokawa/virtua). TanStack Virtual: React/Vue/Svelte/Solid/Lit/Angular adapters, combining axes for grids — [TanStack Virtual](https://tanstack.com/virtual/latest/docs/introduction).
- Microsoft advises light initial payload, lazy-loading images, CSS rather than JS animation, and not disabling GPU — [WebView2 performance](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/performance).

### Inferences
- In a local-asset app the 10 KB vs 50 KB difference matters little for download; it matters for reopen time if the window is destroyed on hide, since JS is parsed on every reopen. Framework JS heap (2-4 MB) is dwarfed by WebView2 processes (~110+ MB) and decoded images.
- Grid recipe: fixed-aspect cells (known size from API metadata, so no reflow), row-virtualised list with `virtua`/TanStack plus small overscan, `<img loading="lazy" decoding="async">` at display-size thumbnails from the custom protocol, dominant-colour/blurhash placeholder, cap decoded images by unmounting off-screen rows, animate only transform/opacity.

### Gaps
- No benchmark found for scrolling large image grids in WebView2 specifically. Solid 2 compatibility with virtua/TanStack not verified.

## 9. Distribution: signing, SmartScreen, Store, updates

### Takeaway
No certificate buys instant SmartScreen trust any more. The free way to avoid warnings is a Store MSIX, but Tauri cannot produce MSIX; a Store EXE listing requires a signed offline installer (+~127 MB). Azure Artifact Signing (~$9.99/month) is closed to individuals outside the US and Canada, leaving an OV certificate at about $150-300/year.

### Cited Findings
- Options table, 29 Oct 2026 — [Microsoft code signing options](https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/code-signing-options): Store MSIX free, re-signed, no warnings, worldwide; Store MSI/EXE must be signed by a trusted CA; Azure Artifact Signing ~$9.99/month, organisations in US/Canada/EU/UK, **individuals US and Canada only**, reputation builds over time; OV $150-300/year, worldwide, key in HSM/token; EV $400+/year, **same SmartScreen behaviour as OV since 2024**; unsigned = strong block. SignPath Foundation: free signing for qualifying open-source projects.
- SmartScreen checks file and certificate reputation; no reputation means a warning — [SmartScreen, Nov 2026](https://learn.microsoft.com/en-us/windows/security/operating-system-security/virus-and-threat-protection/microsoft-defender-smartscreen/).
- Smart App Control on Windows 11 blocks unknown unsigned apps outright with no per-app bypass — [Tauri signing docs](https://v2.tauri.app/distribute/sign/windows/).
- Store developer registration: free for individuals since Sep 2025 — [Windows Developer Blog](https://blogs.windows.com/windowsdeveloper/2025/09/10/free-developer-registration-for-individual-developers-on-microsoft-store/); company $99 fee removed May 2026 — [Windows Developer Blog](https://blogs.windows.com/windowsdeveloper/2026/05/07/publish-to-microsoft-store-as-a-company-now-with-free-registration-and-faster-onboarding/) (search extracts; the sign-up FAQ says some markets/entry points may still charge).
- Store EXE/MSI rules: signed binary and all PE files, versioned immutable HTTPS URL that you host, silent install, standalone offline installer — [MSI/EXE requirements](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msi/app-package-requirements).
- "Currently Tauri only generates EXE and MSI installers"; Store builds must use `offlineInstaller` WebView2 mode and be signed, and app must handle its own updates — [Tauri Microsoft Store](https://v2.tauri.app/distribute/microsoft-store/).
- For MSIX via App Installer, WebView2 can be declared as an external dependency — [WebView2 distribution](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/distribution).

### Inferences
- Realistic path: GitHub/website NSIS download (~1-5 MB) with the Tauri updater first; sign with OV (or SignPath if open source; Artifact Signing only if eligible); later, hand-package an MSIX for the Store to get free signing and no SmartScreen warning (extra work outside Tauri's bundler; per-user autostart and updater would need to be handled differently in MSIX).
- The updater's signature key is separate from Authenticode; both are needed.

### Gaps
- No source fetched for a supported Tauri-to-MSIX workflow; Artifact Signing price taken from Microsoft's table, not the pricing page (its amounts were blank).

## 10. Recommendation: evidence for the pick and the runner-up

### Takeaway
**Recommended: Tauri 2.12 + Rust core + Svelte 5, with no window at autostart and destroy-on-hide.** It is chosen for installer size, UI freedom and Windows 10 coverage, not for RAM with the window open (equal to Electron) or cold start (unproven, possibly slower). **Runner-up: Slint** (same Rust core, lowest measured RAM), with **WPF on .NET 10/11** the best alternative if a mature native toolkit matters more than payload size.

### Cited Findings
- Tauri for: 1.05 MB installer vs 106 MB Electron — [Dawloom](https://dawloom.com/blog/tauri-vs-electron-vs-wpf-benchmark/); 0 WebView2 processes when destroyed — [BettrFTHR PR 8](https://github.com/H4ck3rPr0G4m3r/BettrFTHR/pull/8); 6 MB private Rust host — [footnote#516](https://github.com/bobcat88/footnote/issues/516); WebView2 maintained on Win10 to Oct 2028 — [Edge](https://learn.microsoft.com/en-us/deployedge/microsoft-edge-supported-operating-systems); official plugins for every need (section 6).
- Tauri against: 142 vs 149 MB private with window open — [Dawloom](https://dawloom.com/blog/tauri-vs-electron-vs-wpf-benchmark/); ~650 ms vs ~195 ms start vs Electron on CI — [Elanis](https://github.com/Elanis/web-to-desktop-framework-comparison); WebView2 memory controls not exposed — [tauri#16618](https://github.com/tauri-apps/tauri/issues/16618); needs MSVC (section 7); a maintainer of a Tauri always-visible dock app calls WebView2 processes "the major drawback of tauri" and has considered native but finds equally smooth animation hard — [bloom#366, Nov 2026](https://github.com/little-systems/bloom/issues/366).
- Slint for: 5.8 MB tray / 10.5 MB with window — [claude-rpc PR 7](https://github.com/inerthel-agi/claude-rpc/pull/7). Against: GPU renderer raised one app to ~155 MB on Windows — [slint#13470](https://github.com/slint-ui/slint/issues/13470); those low numbers are software-rendered settings UIs; no tooltips and lost self-update in that migration; royalty-free desktop licence requires the AboutSlint widget or a public badge — [Slint terms](https://slint.dev/terms-and-conditions).
- WPF for: 16.7 MB private in tray, 74 MB with window — [benchbar](https://github.com/askysh/benchbar/pull/70), [Dawloom](https://dawloom.com/blog/tauri-vs-electron-vs-wpf-benchmark/). Against: 56-60 MB zipped self-contained or a .NET Desktop Runtime prerequisite — [Dawloom](https://dawloom.com/blog/tauri-vs-electron-vs-wpf-benchmark/); .NET 10/11 supports Win10 only for LTSC/Enterprise — [Microsoft](https://learn.microsoft.com/en-us/dotnet/core/install/windows).

### Inferences
Scorecard (my judgement from the evidence above; rank 1 = best, 5 = worst)

| Stack | Payload | Tray RAM | RAM, window open | Custom UI / grid ease | Win10+11 officially OK | Main reason |
|---|---|---|---|---|---|---|
| Tauri + destroy-on-hide | 1 | 2 | 4 | 1 | yes | small + web UI; RAM only good when hidden |
| Slint | 1-2 | 1 | 1 (software renderer) | 3 | yes (Rust) | lightest; image-heavy GPU UI unproven |
| WPF | 4 (or runtime) | 2 | 2 | 3 | Win10 consumer not in support list | mature native |
| WinUI 3 | 5 | 3-4 | unknown | 2 (native Win11 look) | same .NET caveat | biggest native payload, more tray RAM than WPF |
| Electron | 5 | 5 unless destroyed | 4 | 1 | yes | 100x installer |
| Wails | 2 | unknown | 5 | 1 | yes | higher RAM than Tauri, no gain |
| Flutter | 2-3 | unknown | unknown | 2 | yes | conflicting figures, slow start on CI |
| Avalonia AOT | 3 | unknown | unknown | 3 | .NET caveat | memory not measured |

- The preference for Tauri survives the test, but for a different reason than usually claimed. It depends on destroy-on-hide; a Tauri app that merely hides its window is as heavy as Electron all day.
- What would change the answer: (a) if the window must stay open or reopen instantly (<150 ms), WPF or Slint wins; (b) if total open-window RAM must be under ~80 MB, Tauri is out; (c) if a Store-only MSIX is the goal, WPF/WinUI fit better; (d) if Build Tools cannot be installed, reconsider.
- Cheap validation before committing (about a day): a Tauri spike and a Slint spike with a 1,000-thumbnail grid, measuring private WS + private bytes over the whole process tree, reopen-to-first-paint after destroy, and scroll frame times, on one Win10 and one Win11 machine.

### Gaps
- No evidence found on image-grid smoothness in Slint, WPF or WebView2 under equal conditions; this is the biggest unknown for "pretty + fast".
- Idle CPU/battery in tray was not measured in any source except that a hidden-but-visible controller costs CPU (0.7-15.7%) while `SetIsVisible(false)` or destroy gives 0%, per [tauri#16618](https://github.com/tauri-apps/tauri/issues/16618).
- Many tray-memory figures come from single-machine PRs; treat as directional.
