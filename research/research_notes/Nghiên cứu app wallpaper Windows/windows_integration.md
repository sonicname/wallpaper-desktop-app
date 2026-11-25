# Windows 10/11 platform integration for a static-image wallpaper app (as of 2026-11-25)

Source labels used below:
- **[OFFICIAL]** Microsoft documentation / Microsoft-owned repo.
- **[CODE]** read directly from a project's source code.
- **[COMMUNITY]** forum, blog, third-party issue or measurement.
- **[LOCAL]** read-only observation by this researcher on 2026-11-25, one PC, Windows 11 26H2 build 26300.10075, one 2560x1440 monitor at 100 % scale. Nothing was modified. N=1, no URL; treat as anecdotal.

Platform baseline: Windows 10 22H2 still matters. The free consumer ESU programme was extended by a year on 2026-06-25 and now ends 2027-10-12 [COMMUNITY/press] — [BleepingComputer](https://bleepingcomputer.com/news/microsoft/microsoft-quietly-extends-free-windows-10-esu-support-to-october-2027), [Pureinfotech](https://pureinfotech.com/windows-10-esu-extended-2027-microsoft-update/) (Tom's Hardware gives conflicting 12/14 Oct dates).

## Setting the wallpaper (IDesktopWallpaper vs SystemParametersInfo vs WinRT)

### Takeaway
Use `IDesktopWallpaper` (works unpackaged, per monitor, Win8+, non-blocking to the caller) on a dedicated COM thread; keep `SPI_SETDESKWALLPAPER` only as fallback or as an optional "fast path, one image for all monitors". Avoid WinRT `UserProfilePersonalizationSettings` for desktop wallpaper in an unpackaged app. Two traps dominate: Windows 11 virtual desktops (no public API covers all of them) and Explorer's JPEG re-encode into `TranscodedWallpaper`.

| | `IDesktopWallpaper` | `SystemParametersInfoW(SPI_SETDESKWALLPAPER)` | WinRT `UserProfilePersonalizationSettings.TrySetWallpaperImageAsync` |
|---|---|---|---|
| Unpackaged Win32/Rust | Yes ("desktop apps only", Win8+) | Yes | Not designed for it: samples use package URIs; evidence below |
| Per monitor | Yes (monitor device path; NULL = all) | No | No |
| Fit mode | `SetPosition` (one global value) | `WallpaperStyle`/`TileWallpaper` registry written beforehand | None |
| Slideshow | `SetSlideshow`, `SetSlideshowOptions`, `AdvanceSlideshow` | No | No |
| Blocks caller | ~1 ms per call (measured) | 1–5 s with `SPIF_SENDCHANGE` (measured) | async |
| All Win11 virtual desktops | No | No (contradicting report) | No source found |

### Cited Findings

**IDesktopWallpaper API**
- Methods: Set/GetWallpaper, GetMonitorDevicePathCount/At, GetMonitorRECT, Set/GetBackgroundColor, Set/GetPosition, Set/GetSlideshow, Set/GetSlideshowOptions, AdvanceSlideshow, GetStatus, Enable. Minimum client Windows 8, "desktop apps only" [OFFICIAL] — [IDesktopWallpaper](https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nn-shobjidl_core-idesktopwallpaper)
- `SetWallpaper(monitorID, path)`: `monitorID` from `GetMonitorDevicePathAt`; NULL = all monitors; full path required [OFFICIAL] — [SetWallpaper](https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nf-shobjidl_core-idesktopwallpaper-setwallpaper)
- Position values: CENTER 0, TILE 1, STRETCH 2, FIT 3 (letterbox, uses background color), FILL 4 (crops), SPAN 5 (one image across all monitors) [OFFICIAL] — [DESKTOP_WALLPAPER_POSITION](https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/ne-shobjidl_core-desktop_wallpaper_position). `SetPosition` has no monitor parameter in the interface list, i.e. it is global — [IDesktopWallpaper](https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nn-shobjidl_core-idesktopwallpaper)
- Enumeration **includes detached monitors that still have an image assigned**; call `GetMonitorRECT`, which returns `S_FALSE` for detached ones [OFFICIAL] — [GetMonitorDevicePathCount](https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nf-shobjidl_core-idesktopwallpaper-getmonitordevicepathcount), [GetMonitorRECT](https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nf-shobjidl_core-idesktopwallpaper-getmonitorrect)
- `GetWallpaper(NULL)` returns `S_OK` if all monitors share one image, `S_FALSE` if different; a monitor with no wallpaper returns success with an empty string (Raymond Chen, 2014-06-02) [OFFICIAL blog] — [The Old New Thing](https://devblogs.microsoft.com/oldnewthing/20140602-00/?p=833)
- `Enable(FALSE)` disables the background; a later `SetWallpaper`/`SetSlideshow` re-enables it [OFFICIAL] — [Enable](https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nf-shobjidl_core-idesktopwallpaper-enable)
- `SetSlideshow(IShellItemArray)` accepts only several files from the same container, or one folder item; anything else fails [OFFICIAL] — [SetSlideshow](https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nf-shobjidl_core-idesktopwallpaper-setslideshow). `SetSlideshowOptions(DSO_SHUFFLEIMAGES, tick_ms)` [OFFICIAL] — [SetSlideshowOptions](https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nf-shobjidl_core-idesktopwallpaper-setslideshowoptions)

**COM activation and threading**
- Microsoft's own sample uses `CoCreateInstance(CLSID_DesktopWallpaper, nullptr, CLSCTX_ALL, …)` and does not discuss apartments [OFFICIAL blog] — [The Old New Thing](https://devblogs.microsoft.com/oldnewthing/20140602-00/?p=833)
- `CLSCTX_INPROC_SERVER` fails with `REGDB_E_CLASSNOTREG (0x80040154)` because the class has only an AppID and no InprocServer32; use `CLSCTX_ALL` or `CLSCTX_LOCAL_SERVER` [CODE, "verified on a real Windows box"] — [deskmakeover wallpaper.rs](https://github.com/nicepkg/deskmakeover/blob/HEAD/crates/dm-windows/src/wallpaper.rs). `CLSCTX_LOCAL_SERVER` is used by [sindresorhus/windows-wallpaper](https://github.com/sindresorhus/windows-wallpaper/blob/HEAD/src/lib.rs) and [Files](https://github.com/files-community/Files/blob/HEAD/src/Files.App/Services/Windows/WindowsWallpaperService.cs). AppID is "Desktop Wallpaper Factory", RunAs Interactive User [COMMUNITY, Oct 2026] — [BingWall #24](https://github.com/mrvonwyl/BingWall/issues/24)
- Apartment advice conflicts: deskmakeover insists on STA ("on an MTA thread these calls silently misbehave") [CODE comment] — [apartment.rs](https://github.com/nicepkg/deskmakeover/blob/HEAD/crates/dm-windows/src/com/apartment.rs); BingWall's research recommends a dedicated MTA worker since calls are out-of-process and no message pump is needed [COMMUNITY] — [BingWall #24](https://github.com/mrvonwyl/BingWall/issues/24). WinDynamicDesktop comments "STA (required!)" only for **IActiveDesktop** [CODE] — [WallpaperApi.cs](https://github.com/t1m0thyj/WinDynamicDesktop/blob/HEAD/src/WallpaperApi.cs)
- On Explorer restart expect RPC errors; re-create the object and retry once (recommendation, inferred by authors) [COMMUNITY] — [BingWall research doc](https://github.com/mrvonwyl/BingWall/blob/research/windows-wallpaper-mechanisms/research/windows-wallpaper-mechanisms.md)

**Measured latency and the accent-colour freeze** (Win11 25H2 10.0.26200, 3440x1440 + 2560x1440, Oct 2026, 3 trials per cell) [COMMUNITY]
- `IDesktopWallpaper::SetWallpaper` returns in ~1 ms; `CoCreateInstance` ~0.9 ms; `SetPosition` ~0.1 ms — [BingWall #23](https://github.com/mrvonwyl/BingWall/issues/23)
- With "Automatically pick an accent color from my background" ON (`AutoColorization=1`): Explorer burned 1.5–3.5 s CPU and its desktop thread hung 0.9–1.6 s per change. OFF: Explorer CPU 0.1–0.25 s, hang ~50–60 ms — [BingWall #23](https://github.com/mrvonwyl/BingWall/issues/23), repeated in [#37](https://github.com/mrvonwyl/BingWall/issues/37)
- Time to first visible change with auto-accent OFF: SPI ~100 ms, IActiveDesktop ~108 ms, IDesktopWallpaper ~465 ms; but SPI blocked the caller 1–5 s because of the `SPIF_SENDCHANGE` broadcast, and `IActiveDesktop::ApplyChanges` returned ERROR_TIMEOUT every time — [BingWall #23](https://github.com/mrvonwyl/BingWall/issues/23)
- Input format (full JPG / prescaled JPG / BMP) and NULL vs per-monitor calls made no measurable latency difference — [BingWall #23](https://github.com/mrvonwyl/BingWall/issues/23), [#28](https://github.com/mrvonwyl/BingWall/issues/28)

**SystemParametersInfo**
- Returns TRUE unless there is an error such as a missing file [OFFICIAL] — [SystemParametersInfoW](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-systemparametersinfow). `SPI_GETDESKWALLPAPER` result never exceeds MAX_PATH — same page.
- `SPIF_SENDCHANGE` broadcast "will hang until that unresponsive window finally resumes responding to messages or is killed". Fixes: worker thread; omit the flag and use `SendMessageTimeout(SMTO_ABORTIFHUNG|SMTO_NOTIMEOUTIFNOTHUNG)`; or, best, `SendNotifyMessage` (Raymond Chen, 2005-03-10) [OFFICIAL blog] — [The Old New Thing](https://devblogs.microsoft.com/oldnewthing/20050310-00/?p=36233)

**WinRT UserProfilePersonalizationSettings**
- `TrySetWallpaperImageAsync(StorageFile)`: new file must have a **different name** than the previous one or the call fails [OFFICIAL] — [TrySetWallpaperImageAsync](https://learn.microsoft.com/en-us/uwp/api/windows.system.userprofile.userprofilepersonalizationsettings.trysetwallpaperimageasync). Official example loads `ms-appx:///Local/…`, a package URI — [class page](https://learn.microsoft.com/en-us/uwp/api/windows.system.userprofile.userprofilepersonalizationsettings)
- WinDynamicDesktop uses this API only in its Store (packaged) build, with `ms-appdata:///local/…`, and falls back to IDesktopWallpaper for per-monitor or external files [CODE] — [UwpHelper.cs](https://github.com/t1m0thyj/WinDynamicDesktop/blob/HEAD/src/UwpHelper.cs)

**Formats**
- WebP as a desktop background via Settings > Personalization > Background or Explorer right-click was added in the 2026-02-24 preview KB5077241 (26200.7922 / 26100.7922, 24H2/25H2), gradual rollout [OFFICIAL] — [KB5077241](https://support.microsoft.com/kb/5077241). Press: before that, WebP could still be set indirectly from Photos — [Neowin](https://www.neowin.net/news/windows-11-will-soon-support-more-desktop-background-formats/)
- AVIF via WIC depends on the HEIF Image Extension and AV1 codec, which "might not be available on all PCs" and come from the Store [OFFICIAL] — [HEIF extension codec](https://learn.microsoft.com/en-us/windows/win32/wic/heif-codec)
- Policy-based wallpaper (Personalization CSP) only accepts jpg/jpeg/png [OFFICIAL] — [Personalization CSP](https://learn.microsoft.com/en-us/windows/client-management/mdm/personalization-csp)
- [LOCAL] WIC decoders registered on 26H2: BMP, GIF, ICO, CUR, JPEG, PNG, TIFF, DNG, WMPhoto, DDS, HEIF, Webp, Raw, JPEG XL.

**JPEG recompression**
- No Microsoft document covers it; all sources are secondary. JPEG sources are re-encoded to `%AppData%\Microsoft\Windows\Themes\TranscodedWallpaper` at 85 % by default; `HKCU\Control Panel\Desktop\JPEGImportQuality` (DWORD, 60–100) changes it [COMMUNITY] — [Winhelponline](https://www.winhelponline.com/blog/increase-desktop-wallpaper-jpeg-image-quality-windows-10/), [Winaero](https://winaero.com/how-to-disable-wallpaper-jpeg-quality-reduction-in-windows-10), [Windows Central](https://www.windowscentral.com/how-disable-image-compression-desktop-wallpapers-windows-10)
- Effect is contested: a WinDynamicDesktop user (Win10 1809, 2020) still saw lower quality with value 100 and only PNG fixed it — [WDD #230](https://github.com/t1m0thyj/WinDynamicDesktop/issues/230); others say 100 only inflates the file to ~10 MB — [The Windows Club](https://www.thewindowsclub.com/disable-windows-10-wallpaper-compression)
- PNG: transcoded copy is nearly the same size as source (not recompressed) [COMMUNITY] — [Winhelponline](https://www.winhelponline.com/blog/increase-desktop-wallpaper-jpeg-image-quality-windows-10/). Dissent: a Windows 8-era thread says PNG larger than ~4800x2700 is resized (not a hard cap; depends on current resolution) — [Microsoft Q&A 2640061](https://learn.microsoft.com/en-us/answers/questions/2640061/any-word-from-microsoft-on-the-horrible-wallpaper?forum=windows-all)
- One unverified user report: a TranscodedWallpaper above ~20 MB gave a black desktop — quoted in search results for [Winaero](https://winaero.com/how-to-disable-wallpaper-jpeg-quality-reduction-in-windows-10/) / gHacks comments
- [LOCAL] On 26H2 with JPEGImportQuality unset: source JPEG 3840x2400, 1,602,752 B on a 2560x1440 monitor became a TranscodedWallpaper of **the same 3840x2400** but 467,431 B (71 % smaller). A `TranscodedWallpaperCache` folder holds hash-named 3840x2400 and 1920x1200 variants. So the re-encode is real and current, and no downscale to monitor size happened. `HKCU\Control Panel\Desktop` also has `MaxMonitorDimension=2560`, `MaxVirtualDesktopDimension=4480`, `TranscodedImageCount`, `TranscodedImageCache`.

**Windows 11 virtual desktops**
- Windows 11 stores a wallpaper per virtual desktop. SPI and `IDesktopWallpaper` reliably change only the **current** desktop; others keep their stored value in `HKCU\…\Explorer\VirtualDesktops\Desktops\{id}\Wallpaper`. Desk research + read-only registry, not a controlled test [COMMUNITY, Oct 2026] — [BingWall #34](https://github.com/mrvonwyl/BingWall/issues/34). Corroborated by [Bing Wallpaper thread 2021–23](https://techcommunity.microsoft.com/discussions/bing/bing-wallpaper-app-issue-on-win-11-with-multiple-desktops/2969683) and [WDD #398](https://github.com/t1m0thyj/WinDynamicDesktop/issues/398); contradicted by [VirtualDesktopAccessor #85](https://github.com/Ciantic/VirtualDesktopAccessor/issues/85) (SPI changed all)
- Only known route to all desktops: undocumented `IVirtualDesktopManagerInternal::UpdateWallpaperPathForAllDesktops(HSTRING)` via `CLSID_ImmersiveShell`. Its IID/vtable changed 4 times since 21H2, twice in cumulative updates (22621.2215, 22631.3085); 23H2 and 24H2 share an IID but have shifted slots, so a build switch is mandatory; stable 24H2 to 25H2 — [BingWall virtual-desktops research](https://github.com/mrvonwyl/BingWall/blob/research/virtual-desktops/research/virtual-desktops.md), [MScholtes/VirtualDesktop](https://github.com/MScholtes/VirtualDesktop)
- A string scan of Microsoft's own Bing Wallpaper app (1.1.469) shows it uses that interface with build-specific definitions, a registry fallback (writes every `Desktops\{id}`) and re-applies after desktop switches — [BingWall #34](https://github.com/mrvonwyl/BingWall/issues/34)
- Per-monitor + virtual desktops is worse: on build 26200 with two 4K displays and 9 desktops, switching desktops can restore a stale single image on both monitors. Proposed workaround (July 2026, experimental): compose one native-resolution bitmap from `GetMonitorRECT` bounds and set it with `Span` for every desktop — [WDD #694](https://github.com/t1m0thyj/WinDynamicDesktop/issues/694)
- Public workaround used by WinDynamicDesktop: an opt-in script syncing desktops — [WDD-scripts experimental](https://github.com/t1m0thyj/WDD-scripts/tree/master/experimental#synchronize-virtual-desktops)

**Spotlight / slideshow / policy / high contrast**
- Current mode can be read from `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Wallpapers\BackgroundType`; no confirmed API to restore Spotlight afterwards [COMMUNITY] — [Microsoft Q&A 1627500](https://learn.microsoft.com/en-us/answers/a/1627500). [LOCAL] value 0 with a plain picture.
- Conflicts do occur: user report of slideshow reverting hours later to a tool-set image — [Directory Opus forum](https://resource.dopus.com/t/set-as-desktop-wallpaper-cannot-be-turned-off/52430); an app's images kept coming back when Windows "Slideshow" remained selected (maintainer hint) — [WDD #462](https://github.com/t1m0thyj/WinDynamicDesktop/issues/462)
- `NoChangingWallPaper` = `HKCU\Software\Microsoft\Windows\CurrentVersion\Policies\ActiveDesktop` REG_DWORD, disables the UI [OFFICIAL, old] — [TechNet Cc975869](https://technet.microsoft.com/en-us/library/Cc975869). Enforcement is leaky: KB 327998 says you must also specify the wallpaper by policy to stop other routes [OFFICIAL, old] — [KB 327998](https://support.microsoft.com/kb/327998). Modern equivalent policy `CPL_Personalization_NoDesktopBackgroundUI` — [Policy CSP](https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-admx-controlpaneldisplay)
- Personalization CSP (Enterprise/Education; Pro only in SharedPC/EDU mode) "also prevents the user from changing the image" [OFFICIAL] — [Personalization CSP](https://learn.microsoft.com/en-us/windows/client-management/mdm/personalization-csp)
- Default high-contrast mode shows a solid black background instead of a wallpaper [COMMUNITY] — [veroniiiica.com](https://veroniiiica.com/choosing-wallpapers-and-backgrounds-with-low-vision)
- 24H2 had a Microsoft compatibility hold (52754008) for third-party wallpaper apps, gradually lifted from 2025-04-11 — [Windows Office Hours](https://techcommunity.microsoft.com/event/windowsevents/windows-office-hours-july-17-2025/4426483/comments/4433510), [The Register, 2025-04-14](https://www.theregister.com/2025/04/14/windows_11_24h2_hold/) (second link cited via BingWall research; not opened here)

**Fade transition**
- Undocumented trick: send message `0x052C` to window class `Progman` before setting, used by WinDynamicDesktop as `EnableTransitions()` [CODE] — [WallpaperApi.cs](https://github.com/t1m0thyj/WinDynamicDesktop/blob/HEAD/src/WallpaperApi.cs). Whether the fade appears depends on the Windows "Animate controls and elements" setting [COMMUNITY, 2021] — [WDD #321](https://github.com/t1m0thyj/WinDynamicDesktop/issues/321). No documented API controls it — [BingWall #24](https://github.com/mrvonwyl/BingWall/issues/24)

### Inferences
- **Recommended design**: one worker thread with COM initialised and the `IDesktopWallpaper` object cached; latest-wins queue; skip monitors whose `GetMonitorRECT` is `S_FALSE`; call `SetPosition` only if different; if every monitor gets the same image call once with NULL. [LOCAL] read calls behaved identically from STA and MTA, so either should work; STA thread without a pump is fine for a short-lived headless process.
- **Set position FILL and supply an image already cropped to the exact monitor pixel size**: then the global position mode is irrelevant, and different monitors can still use different crops.
- **Avoiding quality loss without touching the user's registry**: write the final, monitor-sized image as PNG and set that. PNG at 1440p–4K stays well below the rumoured 20 MB trouble zone for most photos, but verify on real 4K photos; fall back to JPEG q≈95–100 for larger ones. `JPEGImportQuality` is a user-global setting; make it opt-in at most.
- **Give each applied file a fresh name** (e.g. content hash). The WinRT docs demand it, and it sidesteps stale-cache behaviour. Keep the currently applied file plus the previous one on disk: `GetWallpaper`, Settings and undo refer to the source path.
- Always decode WebP/AVIF/JXL yourself and hand Windows PNG/JPEG: Win10 22H2 and un-updated Win11 cannot be assumed to accept them.
- Reading `AutoColorization` and showing a hint ("changing wallpaper will briefly freeze Explorer") is safer than changing it. It makes a strong case for **not** rotating often, and for not applying in the middle of gaming.
- Virtual desktops for a lightweight v1: apply to current desktop; optionally re-apply when the desktop changes; treat the undocumented interface as out of scope or opt-in.
- Detect Spotlight (BackgroundType) at enable time, warn once, and expect SetWallpaper to switch the mode to Picture. Store previous state (path per monitor, position, colour) before the first change so "restore my old wallpaper" works; `GetStatus` tells you if a slideshow is active, but Spotlight cannot be restored.
- Uncompiled sketch for `windows` 0.62 (features `Win32_Foundation, Win32_System_Com, Win32_UI_Shell`):
```rust
CoInitializeEx(None, COINIT_MULTITHREADED).ok()?;
let dw: IDesktopWallpaper = CoCreateInstance(&DesktopWallpaper, None, CLSCTX_ALL)?;
for i in 0..dw.GetMonitorDevicePathCount()? {
    let id = dw.GetMonitorDevicePathAt(i)?;          // PWSTR -> CoTaskMemFree
    let r = dw.GetMonitorRECT(PCWSTR(id.0))?;        // pitfall: S_FALSE maps to Ok in Rust!
    if r.right <= r.left { continue; }               // so treat empty RECT as detached, or call the vtable for the raw HRESULT
    dw.SetWallpaper(PCWSTR(id.0), &HSTRING::from(path))?;
}
```
- .NET equivalents: same COM interface through CsWin32 (`DesktopWallpaper`, `IDesktopWallpaper`, as Files does) or `[ComImport]` GUID `B92B56A9-8B55-4E14-9A89-0199BBB6F93B`, class `C2CF3110-460E-4FC1-B9D0-8A1C0C9CC4BD` ([LOCAL] verified to activate).

### Gaps
- No official statement on max dimensions/file size, on JPEG recompression, or on PNG/WebP/AVIF acceptance by `SetWallpaper` itself (the WebP KB only names the UI routes).
- No source confirms whether `SetWallpaper`/SPI honour `NoChangingWallPaper`; needs a test VM. Same for what RECT is filled in on `S_FALSE` (the empty-RECT check above is unverified), and behaviour of `SetWallpaper` under an active contrast theme.
- Exact apartment requirement is undocumented.
- Whether a Windows 11 26H2 build changed the virtual desktop vtable: not found.

## Lock screen

### Takeaway
An unpackaged app can set the current user's lock screen without admin via WinRT `LockScreen.SetImageFileAsync`, as several unpackaged projects do, though reliability reports are mixed. Registry/CSP routes need admin and lock the setting: avoid them.

### Cited Findings
- `Windows.System.UserProfile.LockScreen.SetImageFileAsync(IStorageFile)`; "not supported when called from x86 Console Apps on x64 machines", so build x64/ARM64 [OFFICIAL] — [SetImageFileAsync](https://learn.microsoft.com/en-us/uwp/api/windows.system.userprofile.lockscreen.setimagefileasync)
- The class page lists no package-identity requirement; Microsoft's list of WinRT APIs not supported or needing identity in desktop apps has no `Windows.System.UserProfile` entry (a search of the page for UserProfile/LockScreen matched only `LockApplicationHost`; the page says the list may not be comprehensive) [OFFICIAL] — [WinRT APIs not supported in desktop apps](https://learn.microsoft.com/en-us/windows/apps/desktop/modernize/desktop-to-uwp-supported-api)
- Unpackaged uses in the wild [CODE]: WinDynamicDesktop desktop build — [LockScreenChanger.cs](https://github.com/t1m0thyj/WinDynamicDesktop/blob/HEAD/src/LockScreenChanger.cs); Rust `windows` crate (10 lines: `StorageFile::GetFileFromPathAsync(..).get()`, `LockScreen::SetImageFileAsync(..).get()`) — [zhousiru/wallpaper-rs](https://github.com/zhousiru/wallpaper-rs/blob/HEAD/src/lockscreen.rs); a Tauri app — [SkyDesk2](https://github.com/angelbests/SkyDesk2/blob/HEAD/src-tauri/src/lockscreen.rs); plain PowerShell — [okieselbach/Intune](https://github.com/okieselbach/Intune/blob/HEAD/Set-Lockscreen.ps1)
- PicView's code comment: it is the same per-user setting the Settings app writes; PersonalizationCSP registry values are a device policy, need admin and show "managed by your organization"; the call is slow, run off-thread; returns failure for undecodable formats or policy-managed lock screens [CODE] — [LockscreenHelper.cs](https://github.com/Ruben2776/PicView/blob/HEAD/src/PicView.Core.WindowsNT/Lockscreen/LockscreenHelper.cs)
- Problems: lock screen stuck on the first image on 26100 (open since 2025-08) — [WDD #651](https://github.com/t1m0thyj/WinDynamicDesktop/issues/651); `TrySetLockScreenImageAsync` worked only the first time (2021) — [WDD #321](https://github.com/t1m0thyj/WinDynamicDesktop/issues/321), consistent with the documented "new file must have a different name" rule — [UserProfilePersonalizationSettings](https://learn.microsoft.com/en-us/uwp/api/windows.system.userprofile.userprofilepersonalizationsettings)
- Community advice: set the lock screen to Picture rather than Spotlight first, else Spotlight may reapply [COMMUNITY] — [Pureinfotech](https://pureinfotech.com/set-lock-screen-image-desktop-wallpaper-windows-11/) / Q&A
- `NoChangingLockScreen` / `ForceDefaultLockScreen` are machine policies that block or override it [OFFICIAL] — [Policy CSP](https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-admx-controlpaneldisplay)

### Inferences
- Offer lock-screen sync as an optional, best-effort feature, unique file name per change, JPEG/PNG only, 10 s timeout, error surfaced but non-fatal. `GetFileFromPathAsync` needs an absolute backslash path.

### Gaps
- No official confirmation for unpackaged use; not executed here (would modify the machine). Interaction with Spotlight on 24H2–26H2 unconfirmed.

## Detecting true resolution, monitors and changes

### Takeaway
Take targets from `IDesktopWallpaper` itself (device path + `GetMonitorRECT`), cross-checked with `EnumDisplaySettings`/`QueryDisplayConfig`, which always report physical pixels. Explicitly set Per-Monitor-V2 in any headless mode, because Tauri enables it only programmatically through its window layer.

### Cited Findings
- `EnumDisplaySettingsW` and `QueryDisplayConfig` do not participate in DPI virtualization; sizes are always physical pixels [OFFICIAL] — [EnumDisplaySettingsW](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-enumdisplaysettingsw), [QueryDisplayConfig](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-querydisplayconfig)
- Other APIs may return virtualized values from unaware/system-aware threads, and this is "not sufficiently documented"; under Per-Monitor awareness the app sees all displays in physical pixels. PMv2 = Win10 1703+ [OFFICIAL] — [High DPI development](https://learn.microsoft.com/en-us/windows/win32/hidpi/high-dpi-desktop-application-development-on-windows)
- Prefer a manifest `<dpiAwareness>PerMonitorV2</dpiAwareness>`; API `SetProcessDpiAwarenessContext` must run before any HWND [OFFICIAL] — [Setting default DPI awareness](https://learn.microsoft.com/en-us/windows/win32/hidpi/setting-the-default-dpi-awareness-for-a-process)
- `QueryDisplayConfig` fails with ERROR_ACCESS_DENIED in a remote session / no console access; use `QDC_ONLY_ACTIVE_PATHS | QDC_VIRTUAL_MODE_AWARE`; path target info carries `rotation` [OFFICIAL] — [QueryDisplayConfig](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-querydisplayconfig), [DISPLAYCONFIG_PATH_TARGET_INFO](https://learn.microsoft.com/en-us/windows/win32/api/wingdi/ns-wingdi-displayconfig_path_target_info). `DEVMODE.dmDisplayOrientation` gives rotation as well — [DEVMODEW](https://learn.microsoft.com/en-us/windows/win32/api/wingdi/ns-wingdi-devmodew)
- Identity: `DISPLAYCONFIG_TARGET_DEVICE_NAME` provides `monitorDevicePath`, friendly name, EDID manufacturer/product IDs (valid only if the flag is set), connectorInstance [OFFICIAL] — [structure](https://learn.microsoft.com/en-us/windows/win32/api/wingdi/ns-wingdi-displayconfig_target_device_name). `EnumDisplayDevices(szDevice, i, EDD_GET_DEVICE_INTERFACE_NAME)` gives the GUID_DEVINTERFACE_MONITOR path — [EnumDisplayDevicesW](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-enumdisplaydevicesw)
- [LOCAL] `IDesktopWallpaper` monitor ID has the same shape: `\\?\DISPLAY#<EDID model>#<instance>&UID<n>#{e6f07b5f-ee97-4a90-b076-33f57bf4eaa7}`.
- PowerToys FancyZones parses that path into device id + instance id and additionally stores the EDID serial number from WMI (`SerialNumberID`) [CODE] — [MonitorUtils.cpp](https://github.com/microsoft/PowerToys/blob/HEAD/src/modules/fancyzones/FancyZonesLib/MonitorUtils.cpp)
- The same physical panel can appear under two device paths (`ACR054F#5&12251521&0&UID4355` and a stale `ACR054F#1&8713bca&0&UID0`) [COMMUNITY observation] — [BingWall research doc](https://github.com/mrvonwyl/BingWall/blob/research/windows-wallpaper-mechanisms/research/windows-wallpaper-mechanisms.md)
- `WM_DISPLAYCHANGE` is sent to top-level windows on resolution change [OFFICIAL] — [WM_DISPLAYCHANGE](https://learn.microsoft.com/en-us/windows/win32/gdi/wm-displaychange); device interface arrivals via [RegisterDeviceNotificationW](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-registerdevicenotificationw)
- Microsoft-documented glitch: after connecting an external display whose resolution differs from the built-in one, the wallpaper can be too small or cut off; workaround is change resolution then Revert (Win10/11) [OFFICIAL] — [Microsoft Support](https://support.microsoft.com/topic/the-wallpaper-might-not-correctly-fit-the-screen-after-connecting-an-external-display-in-windows-6fdc245b-28f5-4333-a205-7e51a2eec593)
- Explorer re-fits the single image itself on hot-plug, so one-image apps need no display listener (desk research, unverified) [COMMUNITY] — [BingWall #33](https://github.com/mrvonwyl/BingWall/issues/33)
- Tauri: `Monitor { name, size: PhysicalSize<u32>, position: PhysicalPosition<i32>, work_area, scale_factor }` [CODE] — [tauri-runtime monitor.rs](https://github.com/tauri-apps/tauri/blob/dev/crates/tauri-runtime/src/monitor.rs). Docs: window/webview APIs report physical pixels, browser APIs and window creation options use logical pixels — [Tauri dpi namespace](https://v2.tauri.app/reference/javascript/api/namespacedpi)
- tao calls `SetProcessDpiAwarenessContext(PER_MONITOR_AWARE_V2)` at run time with fallbacks [CODE] — [tao dpi.rs](https://github.com/tauri-apps/tao/blob/dev/src/platform_impl/windows/dpi.rs), while tauri-build's default manifest contains only the Common Controls dependency, no dpiAwareness [CODE] — [windows-app-manifest.xml](https://github.com/tauri-apps/tauri/blob/dev/crates/tauri-build/src/windows-app-manifest.xml)

### Inferences
- **Headless pitfall**: a Tauri exe started with `--rotate` that exits before building the event loop is DPI-unaware. Ship a custom manifest with PerMonitorV2, or call `SetProcessDpiAwarenessContext` first thing in `main`. Any answer for the GetMonitorRECT-vs-awareness question could not be tested (local machine is 100 % scale).
- In the WebView never use `screen.width * devicePixelRatio`: fractional scales round and only the current monitor is visible. Expose a Rust command returning the physical list.
- `GetMonitorRECT` is already rotated (portrait shows height > width), matching what Explorer paints. Do not swap again using `dmDisplayOrientation`.
- Monitor key: use device path for `SetWallpaper`, but store user settings (favourite folder, per-monitor rules) under EDID model + serial, falling back to device path; re-resolve on each run. Identical monitors with no or duplicate serial must be separated by instance id.
- Change handling: debounce `WM_DISPLAYCHANGE` + `WM_DEVICECHANGE`/`WM_DPICHANGED` by 1–2 s (docks fire bursts), recompute the target set, re-apply only when a monitor's size/orientation really changed and no correctly sized image exists. A headless-only app simply re-evaluates on each scheduled run, plus at unlock if desired.
- HDR: desktop wallpaper is ordinary SDR content; convert wide-gamut/HDR sources to 8-bit sRGB. (Not source-verified.)
- .NET: `EnumDisplaySettings` via P/Invoke; WinForms `Screen.Bounds` only with `HighDpiMode.PerMonitorV2`.

### Gaps
- No authoritative statement on device-path stability across ports/docks/GPU driver updates.
- HDR/Auto Color Management effect on wallpaper not researched in depth; no source found.
- The reliability of `MaxMonitorDimension` registry values is undocumented.

## Choosing and preparing images

### Takeaway
Rank by crop loss and upscale factor computed against physical pixels, per monitor. Pre-crop and downscale to the exact size in-app (Lanczos3 takes ~14 ms for a 16 MP image with AVX2). Use cheap heuristic smart-crop (smartcrop2) only when crop loss is material.

### Cited Findings
- FILL crops to avoid letterboxing, FIT letterboxes with the background colour, SPAN covers all attached monitors with one image [OFFICIAL] — [DESKTOP_WALLPAPER_POSITION](https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/ne-shobjidl_core-desktop_wallpaper_position)
- Pre-scaling gave no apply-latency benefit (3440x1440+2560x1440; UHD source; tested under auto-accent on) [COMMUNITY] — [BingWall #28](https://github.com/mrvonwyl/BingWall/issues/28)
- [LOCAL] Explorer kept a 3840x2400 transcoded copy for a 2560x1440 screen, so it does not shrink modestly oversized images for you.
- `fast_image_resize` RGB8 4928x3279 to 852x567 on Ryzen 9 5950X: Lanczos3 13.5 ms (AVX2), 15.9 ms (SSE4.1), vs 176 ms `image` crate; Bilinear 3.55 ms [project's own benchmark, fir 6.0.1] — [benchmarks-x86_64.md](https://github.com/cykooz/fast_image_resize/blob/main/benchmarks-x86_64.md). It does not convert to linear light; mappers are provided — [README](https://github.com/cykooz/fast_image_resize)
- smartcrop.js: < 20 ms for a square crop of a 640x427 image on an i7 [project claim] — [smartcrop.js](https://github.com/jwagner/smartcrop.js). Rust port `smartcrop2` 0.4.0 (2025-03-04, MIT) is a maintained fork of the unmaintained GPL-3.0 `smartcrop` 0.1.0 (2018), adds black-border removal — [smartcrop.rs](https://codeberg.org/ThetaDev/smartcrop.rs), [crates.io](https://crates.io/crates/smartcrop2)
- Combining different images into one spanned virtual-desktop bitmap handles negative coordinates and mixed DPI since `GetMonitorRECT` is in physical pixels (author's claim) [COMMUNITY] — [WDD #694](https://github.com/t1m0thyj/WinDynamicDesktop/issues/694)

### Inferences
Formulas (arithmetic, no source needed). Monitor `Mw x Mh`, ratio `Rm=Mw/Mh`; image `Iw x Ih`, ratio `Ri`.
- Fill scale `s = max(Mw/Iw, Mh/Ih)`; `s > 1` means upscaling (blur).
- Crop loss `L = 1 - min(Rm/Ri, Ri/Rm)`.
- Score, e.g. `score = 1.0*L + 1.5*max(0, log2(s)) - preference`; hard reject `s > 1.15` or `L > 0.30` (defaults to tune); tie-break towards the smallest adequate image (less download).

| Image on monitor | Crop loss |
|---|---|
| 16:9 on 16:10 | 10 % |
| 3:2 on 16:9 | 15.6 % |
| 4:3 on 16:9 | 25 % |
| 16:9 on 21:9 (3440x1440, 2.389) | 25.6 % |
| 16:9 on 32:9 | 50 % |
| 16:9 on portrait 9:16 | 68.4 % |

- Tiers: (1) exact resolution; (2) same ratio within ±2 %, at least monitor size; (3) ratio tolerance ±12 % at least size; (4) any with `L<=0.30`; (5) FIT with blurred/solid background for extreme cases (32:9, portrait) instead of refusing. Query provider with `at least` + ratio filter per distinct monitor shape, so 32:9 and portrait users do not exhaust a 16:9 pool.
- Different monitors: group by `(w,h)`; one candidate per group unless the user wants different images per screen.
- Decoded RGBA size = `w*h*4`: 1440p 14.7 MB, 4K 33.2 MB, 8K 132.7 MB. Downscaling 8K to 4K cuts decoded pixels 4x. That certainly reduces this app's peak memory and disk use; whether it reduces resident Explorer/DWM memory is plausible but unsourced.
- Pipeline: decode (with EXIF orientation and ICC to sRGB) -> crop -> resize once -> encode PNG -> atomic rename -> set. Avoid double JPEG encoding.
- Focal point: prefer provider metadata or user-set focal point stored per image; else smartcrop on a ~256–512 px thumbnail; constrain vertical offset for portraits. Skip ML saliency/face models to stay lightweight.

### Gaps
- No primary source on Explorer/DWM memory vs wallpaper size.
- No published user study on acceptable crop loss; thresholds are engineering defaults.

## Daily random scheduling and autostart

### Takeaway
Lightest reliable option: a per-user Task Scheduler task (no admin, no resident process) launching `app.exe --rotate` with a daily trigger + `StartWhenAvailable` + logon/unlock triggers scoped to the same user, where the program itself decides idempotently whether today's change is due. A tray timer is the simpler alternative if the app is resident anyway.

### Cited Findings

**Permissions (non-admin)**
- MS-TSCH registration matrix: Non-admin + Self + "running as logged-on user" = allowed, no password. Logon or session-change triggers by a non-admin: **allowed only if the trigger specifies the same user as the registering one**; another user or *nothing specified* is not allowed (E_ACCESSDENIED) [OFFICIAL protocol spec, page updated 2024-10-30] — [MS-TSCH Task Registration Security Checks](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-tsch/d90eccec-83e9-450c-b144-17633c47be9a)
- This explains `schtasks /sc onlogon` failing for standard users: that form schedules for "whenever **any** user logs on" [OFFICIAL] — [schtasks create](https://learn.microsoft.com/en-us/windows-server/administration/windows-commands/schtasks-create). Note the same page elsewhere says "Only Administrators can schedule tasks", which contradicts MS-TSCH and [Security Contexts for Tasks](https://learn.microsoft.com/en-us/windows/win32/taskschd/security-contexts-for-running-tasks)
- From a low-privilege process you can register `TASK_RUNLEVEL_LUA` but not `HIGHEST`; a non-admin registering for own account with interactive or S4U logon needs no password; users may read/update/delete tasks they created [OFFICIAL] — [Security Contexts for Tasks](https://learn.microsoft.com/en-us/windows/win32/taskschd/security-contexts-for-running-tasks)
- `TASK_LOGON_INTERACTIVE_TOKEN` runs only in an existing interactive session; S4U has no network access [OFFICIAL] — [Principal.LogonType](https://learn.microsoft.com/en-us/windows/win32/taskschd/principal-logontype)

**Settings that matter**
- `StartWhenAvailable` default false; late tasks are queued and started after a **default delay of 10 minutes**. The API page says it applies only to time-based tasks "with an end boundary or … set to repeat infinitely"; the schema page just says "applies only to timed tasks" [OFFICIAL, inconsistent] — [ITaskSettings::StartWhenAvailable](https://learn.microsoft.com/en-us/windows/win32/api/taskschd/nf-taskschd-itasksettings-get_startwhenavailable), [schema](https://learn.microsoft.com/en-us/windows/win32/taskschd/taskschedulerschema-startwhenavailable-settingstype-element)
- `DisallowStartIfOnBatteries` **default is True**: laptops silently skip unless set false [OFFICIAL] — [schema](https://learn.microsoft.com/en-us/windows/win32/taskschd/taskschedulerschema-disallowstartifonbatteries-settingstype-element)
- `RunOnlyIfNetworkAvailable` default false [OFFICIAL] — [schema](https://learn.microsoft.com/en-us/windows/win32/taskschd/taskschedulerschema-runonlyifnetworkavailable-settingstype-element)
- Default priority 7 = BELOW_NORMAL process class, meant for background tasks [OFFICIAL] — [TaskSettings.Priority](https://learn.microsoft.com/en-us/windows/win32/taskschd/tasksettings-priority)
- Logon triggers support `Delay`; when the Task Scheduler service starts, logon tasks matching logged-on users are run [OFFICIAL] — [LogonTrigger](https://learn.microsoft.com/en-us/windows/win32/taskschd/logontrigger). `SessionStateChangeTrigger` (e.g. unlock) has `UserId` and `Delay` — [schema](https://learn.microsoft.com/en-us/windows/win32/taskschd/taskschedulerschema-sessionstatechangetrigger-triggergroup-element)
- Time zone/DST: `StartBoundary` without offset or `Z` uses the local computer's time zone and DST; with `Z`/offset it is absolute ("Synchronize across time zones"), and users report one-hour shifts after DST [OFFICIAL + COMMUNITY] — [StartBoundary schema](https://learn.microsoft.com/en-us/windows/win32/taskschd/taskschedulerschema-startboundary-triggerbasetype-element), [Q&A 790592](https://learn.microsoft.com/en-us/answers/questions/790592/windows-task-scheduler-synchronize-accross-time-zo)

**WinRT background tasks**: Windows App SDK `BackgroundTaskBuilder` requires MSIX; Microsoft itself points unpackaged apps to Task Scheduler [OFFICIAL] — [Background tasks](https://learn.microsoft.com/en-us/windows/apps/windows-app-sdk/applifecycle/background-tasks). `StartupTask` likewise needs a package — [StartupTask](https://learn.microsoft.com/en-us/uwp/api/windows.applicationmodel.startuptask)

**Resident-process signals**
- `PBT_APMRESUMEAUTOMATIC` is sent on every resume; `PBT_APMRESUMESUSPEND` follows only if user-triggered [OFFICIAL] — [WM_POWERBROADCAST](https://learn.microsoft.com/en-us/windows/win32/power/wm-powerbroadcast). Windowless: `RegisterSuspendResumeNotification(DEVICE_NOTIFY_CALLBACK)`, Win8+ — [docs](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-registersuspendresumenotification)
- Unlock: `WTS_SESSION_UNLOCK` (0x8) — [WM_WTSSESSION_CHANGE](https://learn.microsoft.com/en-us/windows/win32/termserv/wm-wtssession-change). Clock change: [WM_TIMECHANGE](https://learn.microsoft.com/en-us/windows/win32/sysinfo/wm-timechange)
- `SHQueryUserNotificationState`: 1 not present (locked/screensaver), 2 busy (full-screen app/presentation), 3 exclusive D3D full screen, 4 presentation mode, 5 accepts, 6 quiet time, 7 store app running [OFFICIAL] — [enum](https://learn.microsoft.com/en-us/windows/win32/api/shellapi/ne-shellapi-query_user_notification_state)
- Battery saver: `SYSTEM_POWER_STATUS.SystemStatusFlag == 1`, notify via `GUID_POWER_SAVING_STATUS` [OFFICIAL] — [SYSTEM_POWER_STATUS](https://learn.microsoft.com/en-us/windows/win32/api/winbase/ns-winbase-system_power_status)
- Metered: `INetworkCostManager::GetCost(&cost, NULL)`; low 16 bits level (0x1 unrestricted, 0x2 fixed, 0x4 variable), flags 0x10000 over limit, 0x40000 roaming, 0x80000 approaching limit; returns ERROR_NO_NETWORK offline [OFFICIAL] — [GetCost](https://learn.microsoft.com/en-us/windows/win32/api/netlistmgr/nf-netlistmgr-inetworkcostmanager-getcost), [NLM_CONNECTION_COST](https://learn.microsoft.com/en-us/windows/win32/api/netlistmgr/ne-netlistmgr-nlm_connection_cost). WinRT equivalent adds `BackgroundDataUsageRestricted` — [ConnectionCost](https://learn.microsoft.com/en-us/uwp/api/windows.networking.connectivity.connectioncost)
- EcoQoS for background work: `SetProcessInformation(ProcessPowerThrottling)` [OFFICIAL] — [Quality of Service](https://learn.microsoft.com/en-us/windows/win32/procthread/quality-of-service)

**Autostart**
- Run key: command line max 260 chars; order indeterminate; no promptness guarantee, system may delay Run and Startup items [OFFICIAL] — [Run and RunOnce](https://learn.microsoft.com/en-us/windows/win32/setupapi/run-and-runonce-registry-keys)
- Startup delay: undocumented `HKCU\…\Explorer\Serialize` `StartupDelayInMSec` and, from 24H2, `WaitForIdleState` (Windows waits for idle); a user with ~30 startup apps went from 150 s to 30 s after zeroing both [COMMUNITY, 2026-11-13] — [Windows Latest](https://www.windowslatest.com/2026/11/13/your-pc-isnt-slow-to-start-windows-11-is-quietly-holding-your-apps-back-on-purpose-and-one-tweak-cut-a-users-wait-from-2-5-minutes-to-30-seconds/). [LOCAL] the Serialize key is absent by default on 26H2.
- `StartupApproved\Run` is undocumented 12-byte binary: first byte even = enabled, odd = disabled (Mozilla) — [WindowsLaunchOnLogin.sys.mjs](https://dxr.mozilla.org/mozilla-central/source/toolkit/modules/WindowsLaunchOnLogin.sys.mjs); 0x02/0x06 enabled, 0x03 disabled + FILETIME; deleting the Run value leaves the StartupApproved entry — [windowsir 2022](https://windowsir.blogspot.com/2022/07/does-autostart-really-mean-autostart.html)

### Inferences
| Option | Resident cost | Admin | Misses handled by | Verdict |
|---|---|---|---|---|
| Task Scheduler + `--rotate` headless | none | No (same-user, LUA, interactive token) | StartWhenAvailable + logon/unlock triggers + own state | Lightest, recommended default |
| Tray process + timer | full app process (do not keep a WebView alive) | No | resume/unlock events | If hotkeys/tray UI wanted |
| Run key + headless "check and exit" | none after logon | No | runs each logon only | Good fallback when tasks are blocked by policy; misses long uptime |
| Windows slideshow (`SetSlideshow` on cache folder, shuffle, tick=86,400,000 ms) | none (Explorer) | No | Explorer | No history/undo/fit logic; fallback idea only |
| WinRT background task | none | No | OS | Needs package identity |

- **Make `--rotate` idempotent**: compute local date (`YYYY-MM-DD` in current time zone, with a user "change at" hour); if `last_applied_date >= today` exit in milliseconds. Every trigger can then fire safely: daily, logon (+30–60 s delay), unlock. This absorbs missed days (apply once, not N times), DST skips/duplicates, time-zone travel and clock changes, and the 10-minute StartWhenAvailable delay. If the stored date is in the future (clock rolled back), clamp to today.
- Suggested task XML: `CalendarTrigger/ScheduleByDay` with offset-less StartBoundary; `LogonTrigger` and `SessionStateChangeTrigger(SessionUnlock)` **both with `<UserId>` set**; `LogonType InteractiveToken`; `RunLevel LeastPrivilege`; `StartWhenAvailable true`; `DisallowStartIfOnBatteries false`; `StopIfGoingOnBatteries false`; `RunOnlyIfNetworkAvailable false` (the next image is already cached); `WakeToRun false`; `ExecutionTimeLimit PT5M`; `MultipleInstancesPolicy IgnoreNew`. Add a far-future EndBoundary if StartWhenAvailable does not fire (docs inconsistency above); test.
- Register through COM (`ITaskService`, `RegisterTask` with XML) rather than spawning `schtasks.exe`: no console flash, works with UserId, no locale date problems. Use a GUI-subsystem exe so the scheduled run shows no console. Fall back to Run key if registration returns E_ACCESSDENIED (managed PCs).
- Separate **apply** (local, offline, instant) from **prefetch** (network; retry next trigger; skip on metered/roaming/battery saver unless allowed).
- Full screen/game: if state is 2, 3 or 4, defer. Headless: exit without updating the date so the next unlock/trigger retries; also consider a repetition (e.g. every 2 h) since no later trigger might occur that day. State 1 (locked) is a good moment to change. Borderless-windowed games may report 2 or 5; do not rely on 3 alone. (Detection coverage unverified.)
- Run via task = task must be removed on uninstall; store it under `\<AppName>\` and repair the exe path on each interactive launch (per-user updaters move the exe).
- .NET: NuGet `TaskScheduler` (dahall) wraps the same COM API.

### Gaps
- Not tested on a standard-user account here; MS-TSCH states SHOULD/MUST for servers, and group policy can restrict it.
- No source for a daily local-time trigger scheduled inside the skipped/duplicated DST hour; avoid 02:00–03:00 default times.
- Modern Standby specifics for Task Scheduler not confirmed.
- Windows slideshow pausing on battery not researched.

## Random without repetition, prefetch, cache, undo

### Takeaway
A persisted shuffle-bag plus history in SQLite/JSON, with one or more prepared (already cropped and encoded) images per monitor shape waiting on disk, makes switching instant and offline-safe. This area is design reasoning; few citable sources exist.

### Cited Findings
- WinDynamicDesktop users explicitly asked for "Random from favorites but different from last" (2024-09) [COMMUNITY] — [WDD #608](https://github.com/t1m0thyj/WinDynamicDesktop/issues/608)
- Setting a same-named file can fail (WinRT) [OFFICIAL] — [UserProfilePersonalizationSettings](https://learn.microsoft.com/en-us/uwp/api/windows.system.userprofile.userprofilepersonalizationsettings); supports unique file names per image
- BingWall's analysis: Explorer redraws from the transcoded copy, but nobody verified whether it ever returns to the original; safe rule "never delete the applied image" [COMMUNITY] — [BingWall hot-plug research](https://github.com/mrvonwyl/BingWall/blob/research/monitor-hotplug/research/monitor-hotplug.md)
- `rusqlite` 0.43.0 (2026-11-15, MIT) is current — [crates.io](https://crates.io/crates/rusqlite)

### Inferences
- **Finite set (favourites/local folder)**: Fisher–Yates shuffle bag, persist order + cursor; on reshuffle forbid the last N (N = min(10, size/2)) from the head. Seed stored so order is reproducible and cheap to recompute when items are added (append to random positions after cursor).
- **Open-ended online source**: random provider query, reject IDs in `seen` (provider id) and near-duplicates by perceptual hash (dHash 64-bit, Hamming <= 6) across providers; cap retries, then widen filters; only then allow the oldest seen item.
- **Daily deterministic variant**: `seed = hash(date, install_id, monitor_group)` gives a stable pick even if the state file is lost.
- **Prefetch**: after each apply, prepare `K=2..3` next images per monitor group (download to `.part`, verify decode and dimensions, crop/resize, encode, atomic rename, mark `ready`). Apply only ever touches `ready` files. If monitor geometry changed, re-render from the stored original (keep originals for the current + queued items only).
- **Cache eviction**: size cap (e.g. 500 MB default) LRU; pinned classes never evicted: applied image(s), previous N for undo (default 10), favourites if "keep offline", queue. Thumbnails use a separate cap.
- **Undo**: history stack of `{applied_at, per-monitor path, position, source id}`; undo re-applies and moves the cursor, "never show again" adds to blocklist (id + pHash + optional tag/author) and advances. Record wallpaper found before first run as entry 0.
- Single-writer: named mutex shared by GUI and headless process (or `tauri-plugin-single-instance` only for the GUI path; headless should take the mutex and exit if busy). SQLite WAL tolerates both.

### Gaps
- No authoritative source for optimal history window, queue depth or cache size; values above are proposals.

## Rust crates and Tauri plugins (checked on crates.io 2026-11-25)

### Takeaway
Pin `windows = "0.62"` (latest published; also what Tauri uses). A breaking 0.100 generation exists for `windows-core` etc. but the umbrella `windows` crate had not been published at 0.100. Skip `wallpaper` and `more-wallpapers`: SPI-only on Windows and dormant.

### Cited Findings
| Crate | Latest | Date | Notes |
|---|---|---|---|
| windows | 0.62.2 | 2025-10-06 | MSRV 1.82 — [crates.io](https://crates.io/crates/windows) |
| windows-core / windows-registry | 0.100.0 | 2026-09-03 | MSRV 1.95 — [crates.io](https://crates.io/crates/windows-core) |
| tauri | 2.12.7 | 2026-11-18 | MSRV 1.90 — [crates.io](https://crates.io/crates/tauri) |
| tao | 0.38.0 | 2026-11-24 | — [crates.io](https://crates.io/crates/tao) |
| tauri-plugin-autostart | 2.7.0 (3.0.0-alpha.3 exists) | 2026-10-01 | — [crates.io](https://crates.io/crates/tauri-plugin-autostart) |
| tauri-plugin-single-instance | 2.5.5 | 2026-10-28 | — [crates.io](https://crates.io/crates/tauri-plugin-single-instance) |
| auto-launch | 0.6.0 | 2026-01-10 | — [crates.io](https://crates.io/crates/auto-launch) |
| wallpaper | 3.2.0 | 2021-07-09 | last push 2024-02, SPI + registry — [source](https://github.com/reujab/wallpaper.rs/blob/HEAD/src/windows.rs) |
| more-wallpapers | 0.4.0 | 2024-03-29 | Windows reports one "Unknow" screen and falls back to single-wallpaper path — [source](https://github.com/LuckyTurtleDev/more-wallpapers/blob/HEAD/more-wallpapers/src/windows.rs) |
| image | 0.25.10 | 2026-03-10 | MSRV 1.88 — [crates.io](https://crates.io/crates/image) |
| zune-jpeg | 0.5.15 (0.5.16-rc3) | 2026-03-26 | used by `image` — [crates.io](https://crates.io/crates/zune-jpeg) |
| image-webp | 0.2.4 | 2025-08-27 | — [crates.io](https://crates.io/crates/image-webp) |
| fast_image_resize | 6.2.0 | 2026-11-20 | MSRV 1.87 — [crates.io](https://crates.io/crates/fast_image_resize) |
| jpeg-encoder | 0.7.1 | 2026-07-27 | used by `image` — [crates.io](https://crates.io/crates/jpeg-encoder) |
| smartcrop2 | 0.4.0 | 2025-03-04 | MIT — [crates.io](https://crates.io/crates/smartcrop2) |
| thumbhash | 0.1.0 | 2023-03-22 | MIT — [crates.io](https://crates.io/crates/thumbhash) |
| kamadak-exif | 0.6.2 | 2026-11-05 | — [crates.io](https://crates.io/crates/kamadak-exif) |
| planif (Task Scheduler) | 1.0.0 | 2023-11-17 | low use (1.7k/90 d) — [crates.io](https://crates.io/crates/planif) |
| display-info | 0.5.10 | 2026-10-22 | — [crates.io](https://crates.io/crates/display-info) |
| jxl-oxide | 0.12.6 | 2026-05-29 | — [crates.io](https://crates.io/crates/jxl-oxide) |

- 0.100 generation: Rust 2024, MSRV 1.95, Win32 metadata regenerated from SDK headers; features/modules grouped by header names instead of category, signatures and ownership types change; release "may initially" omit `windows` and `windows-sys` [OFFICIAL] — [windows-rs #4867](https://github.com/microsoft/windows-rs/issues/4867). Current master README already shows `windows = "0.100"` though crates.io has 0.62.2 — [README](https://github.com/microsoft/windows-rs/blob/master/crates/libs/windows/readme.md). Microsoft suggests focused crates or `windows-bindgen`; the umbrella crate is fine for binaries — [docs/readme](https://github.com/microsoft/windows-rs/blob/master/docs/readme.md)
- Tauri depends on `windows = "0.62"` [CODE] — [tauri Cargo.toml](https://github.com/tauri-apps/tauri/blob/dev/crates/tauri/Cargo.toml)
- `image`'s AVIF **decode** needs `avif-native` (`dav1d` C library); the default `avif` feature is only the `ravif` encoder [CODE] — [image Cargo.toml](https://github.com/image-rs/image/blob/main/Cargo.toml)
- tauri-plugin-autostart wraps `auto-launch`, uses `current_exe()` and, in the part read, sets no Windows enable mode (default `Dynamic`) [CODE, default branch HEAD, may differ from 2.7.0] — [lib.rs](https://github.com/tauri-apps/plugins-workspace/blob/HEAD/plugins/autostart/src/lib.rs). `auto-launch` default mode `Dynamic` tries HKLM first, then HKCU on access denied; also rewrites `StartupApproved\Run` to enabled, i.e. overrides the user's Task Manager choice; `is_enabled` checks both [CODE] — [windows.rs](https://github.com/zzzgydi/auto-launch/blob/HEAD/src/windows.rs). Command-line quoting for paths with spaces was improved on 2026-09-16, **after** the 0.6.0 release — [commit history](https://github.com/zzzgydi/auto-launch/commits/HEAD/src/windows.rs)
- Task Scheduler COM from Rust in a Tauri-like app: example — [Seelen-UI task_scheduler.rs](https://github.com/eythaann/Seelen-UI/blob/HEAD/src/service/task_scheduler.rs)
- Rust `IDesktopWallpaper` examples: [sindresorhus/windows-wallpaper](https://github.com/sindresorhus/windows-wallpaper/blob/HEAD/src/lib.rs), [Seelen-UI](https://github.com/eythaann/Seelen-UI/blob/HEAD/src/service/native_wallpaper.rs), [komorebi](https://github.com/LGUG2Z/komorebi/blob/HEAD/komorebi/src/windows_api.rs)

### Inferences
- Call the plugin's `enable()` only on explicit user action, never on every launch. If running elevated (per-machine installer's "run now"), `Dynamic` may write HKLM; consider using `auto-launch` directly with `WindowsEnableMode::CurrentUser`, or ~20 lines with `windows-registry`.
- Until `auto-launch` > 0.6.0 is out, verify the Run value is quoted for paths with spaces; keep total <= 260 chars.
- Wrap all Win32 access in one module so the 0.62 to 0.100 migration stays contained.
- `image` with `default-features=false, features=["jpeg","png","webp"]` + `fast_image_resize` keeps the binary small; use WIC for AVIF/HEIC/JXL only if the OS has the codec.

### Gaps
- Not verified whether the published tauri-plugin-autostart 2.7.0 pins auto-launch 0.6 or a newer git version.

## Thumbnail grid performance

### Takeaway
Let providers' own thumbnail URLs feed the grid; for local files decode at reduced scale (WIC `IWICBitmapSourceTransform` for JPEG), cache as small JPEG/WebP in an app cache, virtualise the list and use a tiny placeholder hash.

### Cited Findings
- `IWICBitmapSourceTransform` lets codecs natively scale/rotate/convert during decode, "will generally offer performance advantages" over post-decode scaling; `GetClosestSize` reports native sizes [OFFICIAL] — [IWICBitmapSourceTransform](https://learn.microsoft.com/en-us/windows/win32/api/wincodec/nn-wincodec-iwicbitmapsourcetransform). Native JPEG codec supports it — [JPEG format overview](https://learn.microsoft.com/en-us/windows/win32/wic/jpeg-format-overview)
- WIC JPEG encoder defaults to 4:2:0 chroma subsampling; set `JpegYCrCbSubsampling` to 4:4:4 for final wallpapers [OFFICIAL] — [JPEG format overview](https://learn.microsoft.com/en-us/windows/win32/wic/jpeg-format-overview)
- `IShellItemImageFactory::GetImage` returns the shell's cached thumbnail; `SIIGBF_THUMBNAILONLY`, `SIIGBF_BIGGERSIZEOK`, `SIIGBF_MEMORYONLY` flags [OFFICIAL] — [GetImage](https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nf-shobjidl_core-ishellitemimagefactory-getimage)
- WinDynamicDesktop moved previews from WPF to SkiaSharp (2025-11) and in 2026-07 fixed hangs when a thumbnail could not be decoded (WebP) [COMMUNITY] — [WDD #672](https://github.com/t1m0thyj/WinDynamicDesktop/issues/672), [#699](https://github.com/t1m0thyj/WinDynamicDesktop/issues/699), [#700](https://github.com/t1m0thyj/WinDynamicDesktop/issues/700): always have a decode-failure path
- `fast_image_resize` Bilinear 16 MP to 852x567: 3.55 ms AVX2, so decode (not resize) dominates [project benchmark] — [benchmarks](https://github.com/cykooz/fast_image_resize/blob/main/benchmarks-x86_64.md)

### Inferences
- Cache thumbs ~480 px wide (covers 240–320 logical px at 150 % scale) as JPEG q75–80 or WebP: roughly 20–50 KB each, so 2,000 thumbs is about 40–100 MB; cap at 100–150 MB LRU (estimates). Key = hash(source id + size + version).
- In Tauri serve via asset/custom protocol with `Cache-Control: immutable`; `<img loading="lazy" decoding="async">`, `content-visibility:auto`, virtualised rows; never base64 over IPC. Use 2–4 decode workers at below-normal priority.
- JPEG DCT scaling (1/2, 1/4, 1/8) cuts memory for 8K local files: 1/8 of 7680x4320 = 960x540, about 2 MB RGBA vs 133 MB (arithmetic).
- Placeholder: thumbhash (about 25–30 bytes) stored in DB, or provider-supplied dominant colour.

### Gaps
- No benchmark found comparing WIC scaled decode with zune-jpeg full decode on current hardware; no Tauri/WebView2 grid benchmark found.
