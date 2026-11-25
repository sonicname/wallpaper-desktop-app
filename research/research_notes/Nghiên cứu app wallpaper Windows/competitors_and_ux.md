# Competitors and UX of Windows 10/11 wallpaper apps (state of 25 Nov 2026)

Method notes for the writer:
- Microsoft Store figures were read on 25 Nov 2026 from the public Store catalogue endpoint (`storeedgefd.dsx.mp.microsoft.com/v9.0/products/{id}`, market = US). Ratings/counts are **US market only**. "Size" is the Store's `ApproximateSizeInBytes` and may cover several architectures, so read it as relative.
- Store reviews were read from the public Store review endpoint (`apps.microsoft.com/api/products/getReviews/{id}`). It returns a limited, non-random sample; star distributions below describe only that sample. Review texts are **user opinion**.
- GitHub stars, releases, asset sizes and download counts: unauthenticated GitHub API, 25 Nov 2026.
- **Reddit returned HTTP 403** to both attempts, so there is no first-hand Reddit evidence; Reddit appears only where a press article quotes it. Elevenforum, The Windows Club, Softpedia and TechSpot pages also returned 403.

## Competitors: technology, size, price, sources, resolution matching, multi-monitor, scheduling, freshness, ratings

### Takeaway
The big free options are either Microsoft's (ad/Bing-driven, no taste controls) or bloated freemium Store apps (100–500 MB, live wallpaper and AI upsells). The small, clean ones are either single-source (Dynamic Theme), dated-looking (John's Background Switcher, JBS), or new hobby projects with about 1–50 GitHub stars.

### Cited Findings

**Built-in / Microsoft**
- Bing Wallpaper: free, Store (US) rating 4.5 from 865 ratings — [Store API](https://storeedgefd.dsx.mp.microsoft.com/v9.0/products/XPFP7F8RL7MB1W?market=US&locale=en-us&deviceFamily=Windows.Desktop). First released in 2020, in the Store since 2024 — [PCWorld, 27 Oct 2025](https://www.pcworld.com/article/2953458/microsofts-free-wallpaper-app-sneakily-diverts-your-clicks-to-bing.html)
- Windows Spotlight on desktop: built in, "displays a new image from around the world every day" — [Microsoft Support](https://support.microsoft.com/en-us/windows/change-your-desktop-background-image-175618be-4cf1-c159-2785-ec2238b433a8)
- Microsoft is retiring its free theme/wallpaper download page in favour of a Store "Themes" section, which Windows Central calls an "unorganized mess" (opinion) — [Windows Central](https://www.windowscentral.com/software-apps/windows-11/microsoft-replaces-free-wallpaper-website-with-an-unorganized-mess-in-the-microsoft-store); Themes area announced Dec 2025 — [Windows Blog, 11 Dec 2025](https://blogs.windows.com/windowsexperience/2025/12/11/your-pc-your-personality-new-themes-in-store-for-you/)

**Mainstream third-party, static or mixed**

| App | Tech | Size | Price / model | Rating (US Store) | Last update | Source |
|---|---|---|---|---|---|---|
| backiee – Wallpaper Studio 10 | Windows Store package (framework not verified) | ~299 MB | Free + IAP; Premium subscription, AI credits | 4.3 (6,581) | 19 Nov 2026 | [Store API](https://storeedgefd.dsx.mp.microsoft.com/v9.0/products/9WZDNCRFHZCD?market=US&locale=en-us&deviceFamily=Windows.Desktop) |
| Dynamic Theme | Windows Store package | ~32 MB | Free (has add-ons) | 4.5 (895) | 9 Jun 2026 | [Store API](https://storeedgefd.dsx.mp.microsoft.com/v9.0/products/9NBLGGH1ZBKW?market=US&locale=en-us&deviceFamily=Windows.Desktop) |
| Wallpaper· (SuperFizz) | not found | ~499 MB | Free + IAP | 4.6 (2,925) | 4 Nov 2026 | [Store API](https://storeedgefd.dsx.mp.microsoft.com/v9.0/products/9MXT89HRQC6P?market=US&locale=en-us&deviceFamily=Windows.Desktop) |
| Desktop HD Wallpapers (Wuhan Net Power) | not found | ~115 MB | Free + IAP | 4.6 (2,290) | 24 Nov 2026 | [Store API](https://storeedgefd.dsx.mp.microsoft.com/v9.0/products/9MTQ0DGS82GJ?market=US&locale=en-us&deviceFamily=Windows.Desktop) |
| Wallpaper – 4K Live Wallpapers (Wuhan Yousigan) | not found | ~262 MB | Free + IAP; markets AI-generated art | 4.6 (1,988) | 24 Nov 2026 | [Store API](https://storeedgefd.dsx.mp.microsoft.com/v9.0/products/9NSH25LT1F9C?market=US&locale=en-us&deviceFamily=Windows.Desktop) |
| Wally – Unsplash Wallpapers | not found | ~10.7 MB | Free, no add-ons | 4.1 (23) | 14 Aug 2026 (added auto changer) | [Store API](https://storeedgefd.dsx.mp.microsoft.com/v9.0/products/9NKCCCWD3RZ0?market=US&locale=en-us&deviceFamily=Windows.Desktop) |
| MyerSplash Photos (Unsplash) | Windows Store package | ~41 MB | no price returned; has add-ons | 4.7 (149) | 23 Dec 2024 (stale) | [Store API](https://storeedgefd.dsx.mp.microsoft.com/v9.0/products/9NBLGGH4VCSN?market=US&locale=en-us&deviceFamily=Windows.Desktop) |
| **WallGaze (ex WallhavenUWP)** | UWP lineage | ~66 MB | Free + IAP "Pro" | 4.4 (8) | 14 Oct 2026 | [Store API](https://storeedgefd.dsx.mp.microsoft.com/v9.0/products/9NJLDD2934RK?market=US&locale=en-us&deviceFamily=Windows.Desktop) |
| John's Background Switcher 6.2 | .NET 4.8 | ~31.1 MB | Free; optional pay-what-you-want licence | not in Store data | 6.2 current | [JBS site](https://johnsad.ventures/software/backgroundswitcher/windows/) |

- backiee: AI wallpaper generator (credits), live wallpapers, weather/time slideshow and widgets all need Premium; sign-in syncs collections across Windows, Android, iPhone and Xbox; search by category, colour, tags — [Store description](https://apps.microsoft.com/detail/9wzdncrfhzcd?hl=en-US)
- Dynamic Theme: daily picture from Bing + Windows Spotlight for wallpaper and/or lock screen, can burn the description into the picture, history, auto-save, pause for a chosen time, notify-only mode, local slideshow with triggers — [Store API description](https://storeedgefd.dsx.mp.microsoft.com/v9.0/products/9NBLGGH1ZBKW?market=US&locale=en-us&deviceFamily=Windows.Desktop). User on limits: "Only changes the picture once a day… If you don't like the picture, you're stuck for 24 hours" (3★, 27 Oct 2026) — [Store reviews](https://apps.microsoft.com/api/products/getReviews/9NBLGGH1ZBKW?gl=US&hl=en-us&pgNo=1&noItems=25&orderBy=3)
- JBS: sources include local, Flickr, Pixabay, Unsplash, Pexels, RSS, Dropbox, OneDrive, SmugMug, **Wallhaven**, Immich; montages/collages/polaroids; v6.2 adds dark mode and an avoid-repeats option; data stored locally, sends a version number on update checks and "can send" anonymised crash logs (opt-in or opt-out not stated) — [JBS site](https://johnsad.ventures/software/backgroundswitcher/windows/). Mac version $8.99 and advertises multi-monitor — [App Store](https://apps.apple.com/app/id907640277)

**Live / dynamic wallpaper (reference points only)**
- Wallpaper Engine: $4.99 US list price, seen at $3.99 on sale in 2026 — [SteamDB](https://steamdb.com/en/tools/steam-price-history/431960-wallpaper-engine); about 1.0M total reviews, "Overwhelmingly Positive", dated 19 Jul 2026 — [Steambase](https://steambase.io/games/wallpaper-engine/reviews) (third-party aggregator)
- Lively Wallpaper: C#/WinUI 3, GPL-3.0, 20,212★, 412 open issues; last GitHub release v2.2.1.0 on 18 Sep 2025 with a 218 MB installer (2.81M downloads) — [GitHub](https://github.com/lively-community/lively/releases); Store 4.6 (19,399), ~548 MB, updated 20 Nov 2026 — [Store API](https://storeedgefd.dsx.mp.microsoft.com/v9.0/products/9NTM2QC6QWS7?market=US&locale=en-us&deviceFamily=Windows.Desktop). A reviewer saw 245.5 MB RAM and said they would be less comfortable with that on a smaller PC — [Windows Central](https://www.windowscentral.com/software-apps/i-finally-tried-lively-wallpaper-to-animate-my-desktop-its-a-dream-for-customization) (undated in search result)
- WinDynamicDesktop: C#, MPL-2.0, 5,068★, v5.7.0 of 16 Jun 2026, x64 setup 67.7 MB, Store 4.5 (348), ~208 MB — [GitHub](https://github.com/t1m0thyj/WinDynamicDesktop/releases), [Store API](https://storeedgefd.dsx.mp.microsoft.com/v9.0/products/9NM8N7DQ3Z5F?market=US&locale=en-us&deviceFamily=Windows.Desktop). Praise: "no watermark, no ads, no subscriptions… doesn't make any resource get consumed unnecessarily" (5★, 9 Aug 2026) — [Store reviews](https://apps.microsoft.com/api/products/getReviews/9NM8N7DQ3Z5F?gl=US&hl=en-us&pgNo=1&noItems=25&orderBy=3)

**New small native/Tauri entrants worth studying**
- dwall (Tauri + Rust, time-of-day themes, 158★, active Nov 2026): settings window is a separate process that runs only while open; the always-on daemon is a 0.63 MB exe with a ~2 MB private working set (3–8 MB committed after applying); daemon needs neither WebView2 nor admin; per-display themes — [GitHub README](https://github.com/dwall-rs/dwall) (author's figures)
- Pane (C#/WinUI 3, preview): per-monitor wallpapers and slideshows, saved "Setups" with Undo, keeps settings of temporarily disconnected monitors — [GitHub](https://github.com/yeawrongperson/Pane)
- Aura (C#/WinUI 3, 1★): 17 sources, several read through site-internal endpoints/pages; the README notes Cloudflare returns 403 to .NET's TLS fingerprint on Pixabay's HTML pages, worked around with `curl.exe` — [GitHub](https://github.com/FahadBinHussain/aura)
- Splashy (Tauri, Unsplash only, 29★): collections/photographers/topics, every 30 min/hourly/daily/custom schedule, photographer credit, but users must supply their **own Unsplash key** — [GitHub](https://github.com/Halfaxas/Splashy)
- bing-wallpaper-now (Tauri, 9★), Wowl (Tauri/Rust: Unsplash, Bing; 3.4 MB installer, v1.3.1 14 Oct 2026, 1★) — [GitHub](https://github.com/qiyuey/bing-wallpaper-now), [GitHub](https://github.com/naderi/wowl)

**Cross-platform inspirations**
- Variety (Linux, Python, GPL-3.0, 1,729★, 0.9.2 on 20 Oct 2026): Wallhaven/Unsplash/Bing/Reddit sources, sort good images from poor ones, tray pause/resume, CLI control; "in maintenance mode" — [GitHub](https://github.com/varietywalls/variety)
- Muzei (Android, ~5.0k★): new artwork daily, wallpaper blurred and dimmed so icons stay legible, double-tap to reveal, public API for third-party sources — [GitHub](https://github.com/muzei/muzei)
- Unsplash Wallpapers (macOS, official, 1.4 MB, last 13 Feb 2025): toolbar picker, daily/weekly, themes or any public collection, history, current monitor by default with apply-to-all. Reviewers ask for minimum resolution, orientation filter, more frequent changes, better multi-monitor; portrait photos blow up blurry — [App Store](https://apps.apple.com/us/app/unsplash-wallpapers/id1284863847?mt=12)
- Irvue (macOS, free + IAP, menu bar): reviewers praise next/previous/download hotkeys; bug: no refresh if launched before Wi-Fi is up — [App Store](https://apps.apple.com/app/irvue/id1039633667) (via search excerpt)
- Plasma 6 Wallhaven plugin (57★, pushed 30 Sep 2026): downloads for offline use — [GitHub](https://github.com/Blacksuan19/plasma-wallpaper-wallhaven-reborn)

### Inferences
- Size gap: Store leaders are 100–500 MB; Tauri Wallhaven clients ship 2–5 MB installers. A "under 10 MB" claim is achievable and visible, though WebView2 RAM is the real test (dwall's split is the strongest precedent found).
- JBS is the most complete free competitor (Wallhaven, scheduling, many sources, local data). Its gap is visual discovery/browsing and modern UI rather than function; this is my reading of its feature list, not a tested claim.

### Gaps
- Splash!/Splashify on Windows, Walli, Backdrops, Pap.er: no reliable current data found. WallpaperHub: no Windows client found; the website was still referenced in 2026 ([Pureinfotech](https://pureinfotech.com/?p=193743) via search excerpt).
- PowerToys: no wallpaper-changing module came up in any result; not checked specifically.
- JBS 6.2 exact release date, Store rating, and Windows per-monitor behaviour not confirmed on the page read.
- Memory use of backiee, Dynamic Theme and JBS: no measured figures found.

## What users complain about most

### Takeaway
Ads and monetisation traps dominate, followed by loss of control (hijacked wallpaper, forced startup, hard to uninstall), instability, AI-generated content, and wrong/low-resolution or badly fitted images, especially on mixed-orientation monitors. Praise consistently repeats "no ads, lightweight, just works".

### Cited Findings
- **Ad as wallpaper (Sept 2026).** Bing Wallpaper set a full-desktop ad for a Harry Potter/Fantastic Beasts film collection on some Windows 10/11 PCs; region-dependent — [Windows Latest, 8 Sep 2026](https://www.windowslatest.com/2026/09/08/microsoft-said-it-would-cut-windows-11-ads-then-put-a-harry-potter-ad-on-your-desktop-if-you-use-bing-wallpaper/), also [PCWorld](https://www.pcworld.com/article/3230487/windows-11s-desktop-is-now-a-full-screen-ad-for-harry-potter-movies.html). It is unclear whether Spotlight was affected (same Windows Latest article).
- Store reviews, Bing Wallpaper: in a sample of 51 reviews dated 6 Sep–23 Nov 2026, 16 were 1★ and 30 were 5★; nearly all the 1★ from 7–8 Sep cite the ad ("Ads are not Wallpaper!", "Immediate uninstall"), one also objects to AI-made backgrounds — [Store reviews](https://apps.microsoft.com/api/products/getReviews/XPFP7F8RL7MB1W?gl=US&hl=en-us&pgNo=1&noItems=25&orderBy=3)
- **Desktop click hijack.** Oct 2025: update made desktop clicks open Bing, on by default, switchable off — [PCWorld](https://www.pcworld.com/article/2953458/microsofts-free-wallpaper-app-sneakily-diverts-your-clicks-to-bing.html). Windows Latest says it was later pulled after backlash (8 Sep 2026 article), then reports it back on 19 Oct 2026: left-clicking the desktop opens Bing.com about once per 24 h, and the toggle is no longer visible; also a pop-up to make Bing the default search in browsers — [Windows Latest, 19 Oct 2026](https://www.windowslatest.com/2026/10/19/microsoft-promised-fewer-windows-11-ads-but-its-wallpaper-app-acts-like-adware-opening-bing-when-you-click-your-desktop/)
- **Bundled behaviours.** Installer asks to set Bing as homepage and default search in Edge, Chrome, Firefox; Nov 2024 criticism ("borderline malware", per Neowin as reported) cited cookies, extension installation, browser settings — [PCWorld](https://www.pcworld.com/article/2953458/microsofts-free-wallpaper-app-sneakily-diverts-your-clicks-to-bing.html); [The Register, 26 Nov 2024](https://www.theregister.com/2024/11/26/bing_wallpaper_app/) (headline only, not read)
- **Microsoft response:** reported as "optional", with "full control over your search or browser settings", but Windows Latest's Sept article frames that quote as about a separate Bing tool and says it had no reply about the ad. Treat as no confirmed on-record comment on the ad — [Windows Latest](https://www.windowslatest.com/2026/09/08/microsoft-said-it-would-cut-windows-11-ads-then-put-a-harry-potter-ad-on-your-desktop-if-you-use-bing-wallpaper/)
- **backiee** (sample of 172 reviews, Sep 2023–Nov 2026: 40×1★, 8×2★, 20×3★, 19×4★, 85×5★). Themes in negative reviews — [Store reviews](https://apps.microsoft.com/api/products/getReviews/9WZDNCRFHZCD?gl=US&hl=en-us&pgNo=1&noItems=25&orderBy=3):
  - Ads: "swamped with ads every few seconds" (30 Jun 2024); ads at first launch, placed near the close button (24 Oct 2025); still full-screen gambling ads after buying lifetime (29 Jun 2025)
  - Paywall: "everything needs a subscription" (16 Sep 2025); live wallpapers paid (multiple, latest 20 Nov 2026); purchase/restore not working (22 Jul, 5 Sep 2025)
  - Stability/speed: "Crashes Computer Constantly" from a paying user who wants a lite version (4 Nov 2025); "SLOOOW!! NOW CRASHES" (24 Oct 2025)
  - AI content: "More than half of the wallpapers are ai" (21 Jul 2026); "Everything upon startup is AI generated" (7 Jun 2026); "AI art is bad!!" (26 Aug 2025)
  - Autostart without consent: "automaticly opens when i turn on my computer" (9 Mar 2026)
  - Praise: most-helpful review (+57, 3 Nov 2026) says the design became much better and slideshow works well; feature request: "turn that search automatically into a slideshow" (25 Nov 2024)
- **Dynamic Theme** (53 reviews): "wallpapers are very low in resolution" (2★, +17, 6 Jul 2024); "Hijacked my wallpaper settings… strip from the registry" (12 Oct 2025); black screen survived uninstall (26 Apr 2026); fails behind VPN (+25, 29 Jun 2023). Praise: "without annoying ads or must-have desktop icons" titled "Ultra lightweight & Does the job" (27 Oct 2026); one user left Bing Wallpaper because a portrait image was scaled on their ultrawide main display after adding a portrait monitor (+20, 6 Apr 2024) — [Store reviews](https://apps.microsoft.com/api/products/getReviews/9NBLGGH1ZBKW?gl=US&hl=en-us&pgNo=1&noItems=25&orderBy=3)
- **Other fit/UI complaints:** Wally users ask to span across monitors (26 Apr 2025), and want categories and resolution (27 Mar 2022); a 1★ says it opens Unsplash without really setting anything (25 Aug 2025) — [Store reviews](https://apps.microsoft.com/api/products/getReviews/9NKCCCWD3RZ0?gl=US&hl=en-us&pgNo=1&noItems=25&orderBy=1). WinDynamicDesktop: settings unreadable in dark mode (25 Feb 2023), dual-display scaling tricky (27 Jan 2025) — [Store reviews](https://apps.microsoft.com/api/products/getReviews/9NM8N7DQ3Z5F?gl=US&hl=en-us&pgNo=1&noItems=25&orderBy=3)

### Inferences
- The Sept–Oct 2026 Bing episodes create a timely opening: users who want "a nice picture daily, no tricks".
- Resource complaints about *static* changers are mostly about freezes, crashes and slowness rather than quoted RAM numbers; "light" must mean fast start and nothing noticeable when idle.
- High averages (4.3–4.6) coexist with angry recent reviews, so averages alone understate dissatisfaction.

### Gaps
- No quantitative ranking of complaints across apps; samples are small and not random. No Reddit threads could be read.

## Existing Wallhaven desktop clients for Windows, and Tauri lessons

### Takeaway
There are many, none dominant. The most-starred Windows client (Electron, Chinese-language) has had no release since June 2024; the live Store client (WallGaze) paywalls auto-rotation; the closest architectural twins (Wallery, WallFlow — both Tauri + React) launched in 2026 with almost no users. Nobody found combines polished UI, per-monitor resolution auto-match, free daily random, and a tiny idle footprint.

### Cited Findings

| Client | Stack | Stars / state | Installer | Notes |
|---|---|---|---|---|
| [leoFitz1024/wallhaven](https://github.com/leoFitz1024/wallhaven) | Electron + Vite, GPL-3.0 | 1,594★; v4.4.7 and last push 14 Jun 2024; 16 open | not found | Chinese; per-screen set, timed online/local rotation, animated; no online favourites |
| [MaLuns/wallhaven](https://github.com/MaLuns/wallhaven) | Go, MIT | 92★; v1.1.3 18 Nov 2026 | 6.2 MB | very few downloads (6 on the Windows setup of that release) |
| [ErKeLost/wallhaven-desktop](https://github.com/ErKeLost/wallhaven-desktop) | **Tauri** + TypeScript, no licence detected | 52★; v0.0.7 13 Sep 2024 | 3.2 MB | dormant; bugs below |
| [luisflorido/wallhaven-desktop](https://github.com/luisflorido/wallhaven-desktop) | Electron + React | 51★; v1.0.5 10 Jun 2024 | – | stale |
| [Spice](https://github.com/dixieflatline76/Spice) | Go; PolyForm **Noncommercial** | 45★; v2.8.1 10 Nov 2026 | 102.7 MB setup | richest feature set, see below |
| [Wallomizer](https://github.com/Bridel-Igor/Wallomizer) | C++, MIT | 20★; release Mar 2024, pushed Aug 2026 | – | slideshow from Wallhaven queries |
| [Wayper](https://github.com/yuukidach/wayper) | TypeScript GUI + CLI, MIT | 16★; v2.0.1 18 Nov 2026 | 121.9 MB | auto-download, rotate, AI control |
| [thaven](https://github.com/ottocsb/thaven) | **Tauri** + Vue | 2★; pushed Jun 2026 | no release | i18n |
| [Wallery](https://github.com/dmitrykirsh/wallery) | **Tauri 2** + React + Tailwind, MIT | 1★; v1.3.1 25 Sep 2026 | **2.3 MB** (21 downloads) | see below |
| [WallFlow](https://github.com/Toastyfil/WallFlow) | **Tauri 2** + React + Tailwind, MIT | 1★; v1.2.0 14 Nov 2026 | **4.6 MB** (2 downloads) | see below |
| [WallGaze](https://apps.microsoft.com/detail/9njldd2934rk?hl=en-US) (WallhavenUWP) | UWP lineage, Store | 4.4 (8 US ratings); 14 Oct 2026 | ~66 MB | **auto-rotation requires in-app purchase** |
| [winwallhaven](https://github.com/saicotic/winwallhaven) | MSIX; WinUI-style samples in repo; GPL-3.0 | 0★ in this repo; pushed Dec 2025 | – | browse/search/set desktop + lock screen; no scheduling or multi-monitor documented; README has unfilled placeholders |

- WallGaze details: Wallhaven login favourites, tray switching/favouriting, gamepad support, Pro = recommendations and batch download, rotation and translation paid; auto-change and live tile "only use safe content" and recommendations exclude sensitive content — [Store API](https://storeedgefd.dsx.mp.microsoft.com/v9.0/products/9NJLDD2934RK?market=US&locale=en-us&deviceFamily=Windows.Desktop)
- winwallhaven's author says existing Windows Wallhaven clients are "all outdated" (developer claim) — [Elevenforum](https://www.elevenforum.com/t/winwallhaven-a-convenient-windows-10-11-client-app-for-wallhaven-cc.40169/latest) (search excerpt; page 403). AlternativeTo lists it as both "open source"/free and proprietary/commercial, a conflict; its only review is by the account apparently behind the app — [AlternativeTo](https://alternativeto.net/software/winwallhaven/about)
- Wallery features: detects monitor resolution; "At least/Exactly" resolution modes, ratio and colour filters; pan/zoom crop; scheduled per-monitor slideshow rules; tray; hero banner + recommendations + endless feed; photo editor; clock/weather widgets; six languages; Windows only — [GitHub](https://github.com/dmitrykirsh/wallery)
- WallFlow features: merges Wallhaven, Wikimedia, Konachan, Safebooru, DeviantArt, Bing, Openverse in one feed; asks Wallhaven for a **fixed** 1920×1080+ rather than the user's screen; applies with `SystemParametersInfo` (a single call; multi-monitor not mentioned); rotates every 5–240 min; safe-content filtering; six dark themes; says it leaves out Unsplash/Pexels/Pixabay because they need keys or are bot-gated — [GitHub](https://github.com/Toastyfil/WallFlow)
- Spice: orientation matched to each monitor before smart crop; face detection; "museum frame" for images that should not be cropped; each display has its own controller and per-display pause; tray shows artist; hotkeys incl. favourite, "delete and block"; pauses during full-screen games, presentations, high GPU, low battery — [GitHub](https://github.com/dixieflatline76/Spice)
- **leoFitz issue tracker = user demand list** — [issues](https://github.com/leoFitz1024/wallhaven/issues?q=is%3Aissue):
  - per-monitor rules, e.g. one landscape + one portrait (#42, Nov 2023); local library should separate landscape/portrait (#52, Mar 2024)
  - groups of wallpapers per context "to avoid embarrassment" (#51, Mar 2024) → safe/work mode
  - tray with previous/next/pause (#35, Oct 2023); option to hide tray icon (#46)
  - rotation silently stops after some time (#38, #39, Oct–Nov 2023); wallpaper disappears when the app exits (#43; the reporter guesses it is overlaid rather than set in Windows)
  - freeze when the local list has 500+ images (#40); white screen if the download folder was deleted (#31); downloads overwrite each other by name (#62); default should be a subfolder of Downloads (#48); clear failed downloads in one go (#70); multi-select download (#72)
  - Enter key in search does nothing (#49); portable version (#56); proxy incl. username/password (#36, #71); Wallhaven blocked in China so images do not load (#34); site outage (#66); virtual desktops cannot be set separately (#37)
- **Tauri-specific lessons:**
  - ErKeLost (Tauri): preview overflows the screen for other ratios (#10), search box deforms (#8), scrollbar squeezes layout on tab switch (#2), "can't use" Aug 2026 unanswered (#11) — [issues](https://github.com/ErKeLost/wallhaven-desktop/issues?q=is%3Aissue)
  - dwall (Tauri): UI cut off under display scaling, fixed by making fixed-size elements responsive — [#500](https://github.com/dwall-rs/dwall/issues/500); background process could not be stopped (#285, 2025); several "cannot use/UI abnormal after update" (#236, #230, #358, #363, #435) — [issues](https://github.com/dwall-rs/dwall/issues?q=is%3Aissue); another app (Tencent Desktop) overriding the wallpaper, resolved only by changing the other app's setting — [#415](https://github.com/dwall-rs/dwall/issues/415)
  - Tauri on WebView2 can use memory similar to Electron, as reported in a Tauri issue — [tauri #5889](https://github.com/tauri-apps/tauri/issues/5889) (via search excerpt; old). Microsoft recommends lowering the memory usage target for a WebView2 not in use — [Microsoft Learn](https://learn.microsoft.com/microsoft-edge/webview2/concepts/performance) (via search excerpt)
- Wallhaven API facts that shape UX: 45 calls/min, 24 results/page, `random` sorting returns a seed to avoid repeats, NSFW needs an API key, user key applies their own settings/filters — [Wallhaven API docs](https://wallhaven.cc/help/api). A keyless call with `sorting=random&atleast=2560x1440&ratios=16x9&purity=100` returned resolution, ratio, colours, size and three thumbnail sizes (tested 25 Nov 2026) — [API](https://wallhaven.cc/api/v1/search?sorting=random&atleast=2560x1440&ratios=16x9&purity=100)

### Inferences
- The API already gives everything needed for automatic per-monitor matching (`atleast` + `ratios` + random seed) without login; competitors mostly expose these as manual filters or a fixed minimum.
- A small number of downloads for equally small Tauri apps shows that "light + Tauri" alone does not win users. Distribution (Store/winget, signing), polish and reliability of rotation are the likely differentiators.
- Spice is the functional benchmark; it carries a non-commercial licence, so do not copy code. Its 100+ MB and dense feature set leave room for a simple product.
- Adopt dwall's split: native tray/scheduler process always on, web UI destroyed when closed.

### Gaps
- Wallery and WallFlow have zero public issues, so no user feedback exists; their RAM use is unmeasured.
- Did not inspect source code quality or try any app.

## What Windows Spotlight and Bing Wallpaper already do for free (the bar to beat)

### Takeaway
They already give zero-setup, daily, high-quality, credited photos. They do not give taste, resolution/orientation control per monitor, browsing at scale, reliable "keep/skip/never again", or trust.

### Cited Findings
- Windows itself: folder slideshow with interval and shuffle, right-click "Next desktop background"; with Picture you can assign a different image per display, **not available with multiple desktops**; slideshow puts a different picture on each screen; Spotlight gives one new image daily — [Microsoft Support](https://support.microsoft.com/en-us/windows/change-your-desktop-background-image-175618be-4cf1-c159-2785-ec2238b433a8). Contradiction: a Microsoft Q&A answer surfaced in search says different simultaneous slideshow images per monitor are unsupported on Windows 11 — trust the Support page but test ([Q&A](https://learn.microsoft.com/en-us/answers/questions/5565688/how-does-windows-11-place-different-pictures-on-ea))
- Per-monitor assignment reportedly does not work with Spotlight or solid colour (user report, 2022–23) — [Tech Community](https://techcommunity.microsoft.com/t5/windows-11/win-11-cannot-change-wallpaper-for-each-monitor/m-p/3820803)
- Spotlight puts a "Learn about this picture" icon on the desktop; a whole genre of removal guides exists, e.g. [Windows Latest, Dec 2024](https://www.windowslatest.com/2024/12/25/permanently-remove-learn-more-about-this-picture-icon-in-windows-11/amp/) and [Ghacks](https://www.ghacks.net/?p=178881), which describe a settings toggle or registry edit. Icon redesigned in early-2026 preview builds — [Neowin](https://www.neowin.net/news/first-preview-build-for-windows-11-in-2026-is-out-with-fixes-for-file-explorer-and-more/)
- Reliability: Spotlight (lock screen) image disappearing, with one user linking it to local accounts plus OneDrive sign-in (Mar 2026, unverified) — [Microsoft Q&A](https://learn.microsoft.com/en-us/answers/questions/5817010/windows-spotlight-image-disappears-again); one image shown forever on two laptops (Oct 2025) — [Microsoft Q&A](https://learn.microsoft.com/answers/a/12268253) (from a search excerpt; exact thread URL not verified)
- Edition gap: a user says Spotlight is missing on the Education SKU (Apr 2024) — [Dynamic Theme reviews](https://apps.microsoft.com/api/products/getReviews/9NBLGGH1ZBKW?gl=US&hl=en-us&pgNo=1&noItems=25&orderBy=3)
- Bing Wallpaper: new image each day and browse images/learn where they are from — [Store API](https://storeedgefd.dsx.mp.microsoft.com/v9.0/products/XPFP7F8RL7MB1W?market=US&locale=en-us&deviceFamily=Windows.Desktop). A user switched away because it has "ZERO configurable settings" for mixed monitors (Apr 2024) — [Dynamic Theme reviews](https://apps.microsoft.com/api/products/getReviews/9NBLGGH1ZBKW?gl=US&hl=en-us&pgNo=1&noItems=25&orderBy=3). Third-party reviewers still advise moving to Spotlight — [Windows Latest, 19 Oct 2026](https://www.windowslatest.com/2026/10/19/microsoft-promised-fewer-windows-11-ads-but-its-wallpaper-app-acts-like-adware-opening-bing-when-you-click-your-desktop/)
- WebP desktop backgrounds reached Beta in build 26220.7653 (early 2026) — [Windows Central](https://www.windowscentral.com/microsoft/windows-11/windows-11-new-features-2026-so-far). Do not rely on it for Windows 10/older 11.

### Inferences
- "Beat list": (1) first wallpaper in under a minute with no account; (2) correct resolution and orientation on *every* monitor; (3) skip now, not in 24 h; (4) no desktop icon, no watermark by default; (5) user's taste (tags/colours/categories); (6) history so a liked image is never lost.
- Microsoft's defaults are stronger on curation: Wallhaven's random feed is uneven and leans to anime/games/people. The daily pick therefore needs quality heuristics (toplist/favourites threshold, category default).

### Gaps
- Exact desktop Spotlight resolution, and whether Windows 11 Spotlight desktop currently has like/dislike, were not confirmed. Windows 10 desktop Spotlight availability not verified.

## UI/UX patterns and design references

### Takeaway
Pattern consensus: image-first grid → immersive preview with metadata → one-click apply, plus a tray menu that does most of the daily work. Strongest references: Muzei (calm daily art, dim/blur, plug-in sources), Unsplash Wallpapers (tiny toolbar app), Pane (monitor-aware preview, Undo), Spice (attribution and per-display tray), Windows 11 Mica guidance.

### Cited Findings
- Grid/browse: responsive grid + full-screen preview with resolution and author + one-click apply (WallFlow) — [GitHub](https://github.com/Toastyfil/WallFlow); hero banner + "recommended" + endless feed, crop with pan/zoom (Wallery) — [GitHub](https://github.com/dmitrykirsh/wallery); "async loading for smooth browsing", keyboard/mouse/touch (winwallhaven) — [GitHub](https://github.com/saicotic/winwallhaven)
- Monitor picker: monitor-aware previews, disconnected displays kept, Undo after switching setups (Pane) — [GitHub](https://github.com/yeawrongperson/Pane); Unsplash applies to the current monitor by default with an all-monitors option — [App Store](https://apps.apple.com/us/app/unsplash-wallpapers/id1284863847?mt=12)
- Tray: requested items are previous / next / pause — [leoFitz #35](https://github.com/leoFitz1024/wallhaven/issues/35); Spice shows the artist live and per-display pause, plus favourite and "delete and block" — [GitHub](https://github.com/dixieflatline76/Spice); WallGaze tray does switch + favourite — [Store API](https://storeedgefd.dsx.mp.microsoft.com/v9.0/products/9NJLDD2934RK?market=US&locale=en-us&deviceFamily=Windows.Desktop)
- Hotkeys: Irvue users praise next/previous/download shortcuts — [App Store](https://apps.apple.com/app/irvue/id1039633667) (search excerpt); Spice lets users disable hotkey groups that conflict, and lists custom keys as still missing — [GitHub](https://github.com/dixieflatline76/Spice)
- Fit for awkward images: face-aware crop, anchor grid, "museum frame" (Spice) — [GitHub](https://github.com/dixieflatline76/Spice)
- Courtesy: pause on games/presentations/low battery (Spice) — [GitHub](https://github.com/dixieflatline76/Spice); pause daily updates for a chosen length of time (Dynamic Theme) — [Store API](https://storeedgefd.dsx.mp.microsoft.com/v9.0/products/9NBLGGH1ZBKW?market=US&locale=en-us&deviceFamily=Windows.Desktop)
- Legibility: Muzei blurs/dims art behind icons, reveal on double tap — [GitHub](https://github.com/muzei/muzei); Unsplash Mac user wants dimmer images in dark mode — [App Store](https://apps.apple.com/us/app/unsplash-wallpapers/id1284863847?mt=12)
- Mica (doc dated 22 Jul 2026): opaque material sampling the wallpaper **once**, for long-lived windows; apply as a single base layer, show it in the title bar, keep a content layer above it; falls back to a solid colour with transparency off, Battery Saver, low-end hardware, inactive window, or Windows build < 22000 (so **never on Windows 10**); High Contrast respected — [Microsoft Learn](https://learn.microsoft.com/en-us/windows/apps/design/style/mica)
- Praise for MyerSplash: "beautiful UI, fluent animations" (editorial) — [Store listing](https://apps.microsoft.com/detail/9nblggh4vcsn?hl=en-US) (search excerpt). Users request dark-mode readability (WinDynamicDesktop, 2023) — [Store reviews](https://apps.microsoft.com/api/products/getReviews/9NM8N7DQ3Z5F?gl=US&hl=en-us&pgNo=1&noItems=25&orderBy=3)

### Inferences
- Suggested core screens: **Today** (current picture per monitor, skip/like/why this picture), **Explore** (justified grid, since Wallhaven returns dimensions and colours so layout can be reserved before images load and tinted placeholders drawn), **Library** (favourites, history, blocked), **Settings**. Preview should draw a desktop mock with taskbar and icons at the user's real monitor layout.
- Mica is a good thematic fit: the window picks up the wallpaper the user just set. Design the Windows 10 fallback first.
- Onboarding: one screen — detected monitors + 3–5 taste chips + "change daily: on" + explicit start-with-Windows choice — then set the first wallpaper.
- Virtualise the grid and test at 500+ items and 100/125/150/200% scaling, since both failures are documented in sibling apps.

### Gaps
- No usability studies of wallpaper apps found. Accessibility evidence (screen reader, keyboard) is nearly absent in competitors' docs; only keyboard/touch/gamepad claims exist.

## User trust and safety: SFW default, privacy, telemetry, attribution

### Takeaway
SFW must be a hard default for anything automatic. Trust is the market's weak spot after Bing Wallpaper's 2025–26 behaviour, so "no account, no telemetry, no ads, clean uninstall" is a marketable feature.

### Cited Findings
- Wallhaven purity: SFW / Sketchy / NSFW; uploaders are told to mark as Sketchy if unsure — [Wallhaven rules](https://wallhaven.cc/rules). NSFW is blocked for guests and needs an API key — [API docs](https://wallhaven.cc/help/api)
- WallGaze 3.11 changelog: auto-change and live tile only safe content, rating follows the user's Wallhaven account settings — [Store API](https://storeedgefd.dsx.mp.microsoft.com/v9.0/products/9NJLDD2934RK?market=US&locale=en-us&deviceFamily=Windows.Desktop). WallFlow filters catalogue queries to safe content — [GitHub](https://github.com/Toastyfil/WallFlow)
- Wallhaven site rules state "No AI generated images" and a minimum of 1280×800 landscape / 800×1280 portrait — [Wallhaven rules](https://wallhaven.cc/rules). This contrasts with AI complaints against backiee and Bing Wallpaper (above).
- Privacy patterns: JBS stores data locally, sends only a version number for update checks, optional anonymised crash logs — [JBS](https://johnsad.ventures/software/backgroundswitcher/windows/); WallFlow uses one local JSON, no accounts — [GitHub](https://github.com/Toastyfil/WallFlow); winwallhaven lists "no tracking", "no registration" — [AlternativeTo](https://alternativeto.net/software/winwallhaven/about)
- Attribution: "All images remain property of their original owners" (Wallhaven footer) — [Wallhaven terms](https://wallhaven.cc/terms). API results carry `source` (often empty, as in the tested result) and the Wallhaven page URL — [API](https://wallhaven.cc/api/v1/search?sorting=random&atleast=2560x1440&ratios=16x9&purity=100). Dynamic Theme users even ask for the Bing watermark/description (24 Nov 2026), and the app makes it optional — [Store reviews](https://apps.microsoft.com/api/products/getReviews/9NBLGGH1ZBKW?gl=US&hl=en-us&pgNo=1&noItems=25&orderBy=3)

### Inferences
- "SFW" tagging is user-made and still includes suggestive-leaning or fandom content; offer default General category (anime/people opt-in), a tag blocklist, "never show again" plus a one-key panic "revert to safe wallpaper".
- "No AI images" can be a positioning line only for Wallhaven-sourced content, phrased as "source policy", because enforcement is not guaranteed.
- Keep the API key in Windows Credential Manager/DPAPI, not plain JSON (own recommendation).

### Gaps
- No evidence found on whether Wallhaven requires a specific attribution format for API apps.

## Monetisation and legal patterns: hobby/free vs commercial

### Takeaway
Hobby apps are free, open source or donation-based; commercial Store apps mix ads, subscriptions and AI credits and earn the worst reviews. Content rights are the real limit: Wallhaven's images belong to third parties, and Unsplash explicitly forbids wallpaper apps.

### Cited Findings
- Free + pay-what-you-want licence: JBS — [JBS](https://johnsad.ventures/software/backgroundswitcher/windows/). Free/open source: Lively, WinDynamicDesktop, Variety (links above). Source-available non-commercial: Spice — [GitHub](https://github.com/dixieflatline76/Spice). One-time purchase: Wallpaper Engine $4.99 — [SteamDB](https://steamdb.com/en/tools/steam-price-history/431960-wallpaper-engine). Freemium with convenience paywall: WallGaze Pro (rotation, batch download) — [Store API](https://storeedgefd.dsx.mp.microsoft.com/v9.0/products/9NJLDD2934RK?market=US&locale=en-us&deviceFamily=Windows.Desktop). Ads + subscription + credits: backiee — [Store](https://apps.microsoft.com/detail/9wzdncrfhzcd?hl=en-US)
- leoFitz's README says its code cannot be used commercially without permission, alongside GPL-3.0 (a tension) — [GitHub](https://github.com/leoFitz1024/wallhaven)
- **Unsplash API guidelines: "You cannot replicate the core user experience of Unsplash (unofficial clients, wallpaper applications, etc.)"**; hotlinking and download-tracking required; attribution with UTM; keys confidential; apps **should not require users to register for their own developer account**; no Unsplash name/logo in the app — [Unsplash Help](https://help.unsplash.com/en/articles/2511245-unsplash-api-guidelines). Conflict: the API Terms treat "setting an Image as background wallpaper" as a reportable download event — [API Terms](https://unsplash.com/api-terms). Demo mode 50 requests/hour, 1,000 after approval — [Unsplash docs](https://unsplash.com/documentation)
- Wallhaven terms page: no clauses found on automated access, third-party apps, API or commercial use; users keep rights in what they post; images remain property of owners — [Wallhaven terms](https://wallhaven.cc/terms) (read by an extraction tool; have a human re-read before commercialising)

### Inferences
- Charging for access to Wallhaven images, or putting ads next to them, is riskier than charging for app features, as images are not licensed by uploaders or Wallhaven to the app. Safest: free, open source, optional donation; no mirroring or rehosting; link back to the Wallhaven page.
- Do not plan Unsplash as source #2 without written approval. Splashy's bring-your-own-key approach conflicts with the Unsplash guideline. Bing/Spotlight images are widely used but via unofficial endpoints; Wikimedia/museum APIs (used by Spice and WallFlow) look like cleaner second sources, subject to their own terms.
- Avoid "Wallhaven" in the product name, and state "not affiliated" as Wallery does.

### Gaps
- Pexels/Pixabay/NASA/Wikimedia terms not reviewed here (adjacent researcher scope). backiee's current subscription price not found (old articles mention a $1.49 ad removal). Not legal advice.

## Conclusion: evidence-backed must-haves, nice-to-haves, and things to avoid

### Takeaway
Gap statement: **a tiny, ad-free, account-free Windows app that shows a beautiful Wallhaven browser when open and almost nothing when closed, and that automatically gives each monitor a safe, correctly sized, fresh image every day.** Individual pieces exist (Spice: fitting; Wallery: Wallhaven + Tauri; dwall: footprint; Muzei: calm UX), but no found product combines them with adoption.

### Cited Findings

**Must have (v1)**
1. Auto-detect every monitor's physical resolution, ratio and orientation; query `atleast` + `ratios`; never upscale — demand: low-res complaint on [Dynamic Theme](https://apps.microsoft.com/api/products/getReviews/9NBLGGH1ZBKW?gl=US&hl=en-us&pgNo=1&noItems=25&orderBy=3), portrait/landscape per monitor [leoFitz #42](https://github.com/leoFitz1024/wallhaven/issues/42), minimum-resolution request on [Unsplash Mac](https://apps.apple.com/us/app/unsplash-wallpapers/id1284863847?mt=12); feasible per [API docs](https://wallhaven.cc/help/api)
2. Daily random that does not stall, with no repeats, catch-up after sleep/offline, and **skip now** — [leoFitz #38/#39](https://github.com/leoFitz1024/wallhaven/issues?q=is%3Aissue) stops silently; "stuck for 24 hours" on [Dynamic Theme](https://apps.microsoft.com/api/products/getReviews/9NBLGGH1ZBKW?gl=US&hl=en-us&pgNo=1&noItems=25&orderBy=3); Wi-Fi-not-ready bug on [Irvue](https://apps.apple.com/app/irvue/id1039633667). Free, unlike [WallGaze](https://storeedgefd.dsx.mp.microsoft.com/v9.0/products/9NJLDD2934RK?market=US&locale=en-us&deviceFamily=Windows.Desktop)
3. Per-monitor apply (one / all), same or different images — built-in has limits ([Microsoft Support](https://support.microsoft.com/en-us/windows/change-your-desktop-background-image-175618be-4cf1-c159-2785-ec2238b433a8)); users ask on [Wally](https://apps.microsoft.com/api/products/getReviews/9NKCCCWD3RZ0?gl=US&hl=en-us&pgNo=1&noItems=25&orderBy=1)
4. Tray: next, previous, like, never again, pause, open — [leoFitz #35](https://github.com/leoFitz1024/wallhaven/issues/35), [Spice](https://github.com/dixieflatline76/Spice)
5. Fast virtualised grid, search (Enter works), filters for category/colour/ratio/resolution/sort, large preview, one-click set — [leoFitz #40/#49](https://github.com/leoFitz1024/wallhaven/issues?q=is%3Aissue), [winwallhaven](https://github.com/saicotic/winwallhaven); within 45 calls/min and 24 per page ([API docs](https://wallhaven.cc/help/api))
6. SFW-only default, always SFW for automatic changes; General category default — [WallGaze precedent](https://storeedgefd.dsx.mp.microsoft.com/v9.0/products/9NJLDD2934RK?market=US&locale=en-us&deviceFamily=Windows.Desktop), embarrassment concern [leoFitz #51](https://github.com/leoFitz1024/wallhaven/issues/51)
7. Local favourites + history, with managed download folder (subfolder, unique names, survives folder deletion) — [leoFitz #48/#62/#31](https://github.com/leoFitz1024/wallhaven/issues?q=is%3Aissue)
8. Native always-on core with UI destroyed when closed — [dwall](https://github.com/dwall-rs/dwall) ~2 MB daemon vs [Lively 245 MB](https://www.windowscentral.com/software-apps/i-finally-tried-lively-wallpaper-to-animate-my-desktop-its-a-dream-for-customization)
9. No ads, no account, no telemetry by default, opt-in autostart, clean uninstall that leaves a sensible wallpaper — [backiee reviews](https://apps.microsoft.com/api/products/getReviews/9WZDNCRFHZCD?gl=US&hl=en-us&pgNo=1&noItems=25&orderBy=3), [Dynamic Theme reviews](https://apps.microsoft.com/api/products/getReviews/9NBLGGH1ZBKW?gl=US&hl=en-us&pgNo=1&noItems=25&orderBy=3), [Bing reviews](https://apps.microsoft.com/api/products/getReviews/XPFP7F8RL7MB1W?gl=US&hl=en-us&pgNo=1&noItems=25&orderBy=3)
10. Uses the native Windows wallpaper (remains after exit), dark/light, works at all DPI scales — [leoFitz #43](https://github.com/leoFitz1024/wallhaven/issues/43), [dwall #500](https://github.com/dwall-rs/dwall/issues/500), [WinDynamicDesktop dark mode](https://apps.microsoft.com/api/products/getReviews/9NM8N7DQ3Z5F?gl=US&hl=en-us&pgNo=1&noItems=25&orderBy=3)
11. Attribution link (Wallhaven page, source, uploader where available) in preview and tray — [Wallhaven terms](https://wallhaven.cc/terms), [Spice](https://github.com/dixieflatline76/Spice)

**Nice to have (later)**
- Crop/anchor and face-aware fit, framed mode for mismatched ratios — [Spice](https://github.com/dixieflatline76/Spice), [Wallery](https://github.com/dmitrykirsh/wallery)
- Saved search → slideshow (user request on [backiee](https://apps.microsoft.com/api/products/getReviews/9WZDNCRFHZCD?gl=US&hl=en-us&pgNo=1&noItems=25&orderBy=3)); other intervals; global hotkeys (off by default, rebindable)
- Pause on full-screen/low battery — [Spice](https://github.com/dixieflatline76/Spice); keep settings for disconnected monitors, Undo — [Pane](https://github.com/yeawrongperson/Pane)
- Wallhaven API key to sync collections and browsing settings — [API docs](https://wallhaven.cc/help/api); lock screen — [Dynamic Theme](https://storeedgefd.dsx.mp.microsoft.com/v9.0/products/9NBLGGH1ZBKW?market=US&locale=en-us&deviceFamily=Windows.Desktop), [winwallhaven](https://github.com/saicotic/winwallhaven)
- Proxy support ([leoFitz #71](https://github.com/leoFitz1024/wallhaven/issues/71)), portable build (#56), dim/blur ([Muzei](https://github.com/muzei/muzei)), Mica on Windows 11 ([Learn](https://learn.microsoft.com/en-us/windows/apps/design/style/mica)), pluggable sources ([Muzei API](https://github.com/muzei/muzei))

**Avoid**
- Any advertising, click interception, default-browser/search prompts — [Windows Latest](https://www.windowslatest.com/2026/10/19/microsoft-promised-fewer-windows-11-ads-but-its-wallpaper-app-acts-like-adware-opening-bing-when-you-click-your-desktop/)
- Paywalling the daily change; AI-generated feeds; startup without consent; full-screen promos — [backiee reviews](https://apps.microsoft.com/api/products/getReviews/9WZDNCRFHZCD?gl=US&hl=en-us&pgNo=1&noItems=25&orderBy=3)
- Permanent desktop icons or watermarks ([Ghacks](https://www.ghacks.net/?p=178881)); overlay-window wallpapers for static images ([leoFitz #43](https://github.com/leoFitz1024/wallhaven/issues/43))
- Unsplash as a source without approval — [guidelines](https://help.unsplash.com/en/articles/2511245-unsplash-api-guidelines); scraping sites without an API — [Aura's Cloudflare trouble](https://github.com/FahadBinHussain/aura)
- Feature sprawl (widgets, clocks, video, weather, AI) in v1 — contrast [Wallery](https://github.com/dmitrykirsh/wallery), [backiee](https://apps.microsoft.com/detail/9wzdncrfhzcd?hl=en-US)
- Fighting other wallpaper managers silently — detect an external change and pause instead ([dwall #415](https://github.com/dwall-rs/dwall/issues/415), "hijacked" reviews of Dynamic Theme)

### Inferences
- Suggested measurable targets (own proposal, not sourced): installer < 10 MB, tray process < 15 MB private memory, UI open < 1 s, first wallpaper < 60 s from launch.
- Wallhaven is a single point of failure (outage #66, blocking #34/#36): cache a small queue of upcoming images (e.g. 5–7 per monitor) so daily change works offline.

### Gaps
- Demand size for a Wallhaven-specific client is unproven: Store/GitHub evidence shows interest (1.6k★ for leoFitz) but tiny counts for all new clients. User interviews or a small beta are advisable.
- Priorities above are my synthesis of small review/issue samples; none come from a survey.
