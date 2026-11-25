# Chọn Tauri, nhưng phải hủy WebView

> **Cập nhật 25/11/2026: đã chốt Slint 1.19 với software renderer, thay cho đề xuất Tauri bên dưới.** Đo trên máy dev cho thấy Slint mở lại cửa sổ trong 97–98 ms (Tauri 383–420 ms) và dùng 12 MB RAM private khi mở (Tauri 170 MB). Build Tools và Windows SDK cũng đã được cài. Số liệu, giới hạn và mã prototype: [benchmarks/slint-vs-tauri](../benchmarks/slint-vs-tauri/README.md). Phần nguồn ảnh, provider, độ phân giải và daily random vẫn còn hiệu lực; các ý về WebView, Svelte và plugin Tauri thì không. App sẽ miễn phí và mã nguồn mở.

**Dùng Tauri 2.12 + Rust core + Svelte 5, với điều kiện WebView bị hủy khi đóng cửa sổ và không được tạo khi autostart.** Lý do chọn là installer khoảng 1 MB (Electron 106 MB), UI web tự do và hỗ trợ chính thức cả Windows 10 lẫn 11. RAM không phải lý do: khi cửa sổ mở, Tauri dùng gần bằng Electron (142 so với 149 MB private), và cold start chưa có bằng chứng là nhanh hơn. Việc đầu tiên phải làm là **cài MSVC Build Tools và Windows SDK**, vì máy dev hiện thiếu cả hai còn Tauri chỉ hỗ trợ chính thức target MSVC. Wallhaven là nguồn đầu tiên tốt: không cần key cho SFW, 45 call/phút, lọc được resolution và ratio. Tuy vậy `seed` không hoạt động trong test trực tiếp, nên app phải tự chống lặp. Để thêm nguồn nhẹ nhàng, host giữ toàn bộ HTTP, rate limit, cache và secret, còn mỗi provider chỉ map request và response. Hãy loại Unsplash và Pexels khỏi roadmap vì điều khoản cấm đích danh wallpaper app. Chưa nên commit stack trước khi một prototype ngắn (ước lượng 2–3 ngày) trả lời được thời gian mở lại cửa sổ sau khi destroy, RAM của host trong tray và độ mượt của grid ảnh.

**Cách đọc nhãn.** Câu có link là phát hiện có nguồn. **[Tự báo cáo]** là số do một dự án tự đo, thường trên một máy, chưa ai tái lập. **[Test 25/11/2026]** là lệnh gọi API không key trong một ngày, mẫu nhỏ. **[Máy dev]** là quan sát read-only trên đúng máy này, không có URL. **[Đề xuất]** là phán đoán kỹ thuật, cần prototype xác nhận.

| Quyết định | Chọn | Độ tin cậy | Điều làm đổi quyết định |
|---|---|---|---|
| Stack | Tauri 2.12.x, destroy-on-hide | Khá, chờ prototype | Phải mở lại dưới 150 ms, hoặc RAM khi mở dưới ~80 MB |
| Phương án dự phòng | Slint (nhẹ nhất), rồi WPF .NET 10 | Trung bình | Grid ảnh của Slint không mượt |
| Nguồn v1 | Wallhaven + thư mục local | Cao | Wallhaven phản đối app bên thứ ba |
| Cơ chế provider | Rust trait compile-in, thêm declarative manifest khi có nguồn REST/JSON thuần thứ hai | Khá | Cần plugin của bên thứ ba không tin cậy |
| Set wallpaper | `IDesktopWallpaper`, ảnh PNG đã crop đúng pixel | Cao về API, chờ P4 cho đa màn hình | Virtual desktop là yêu cầu bắt buộc |
| Daily random | Hàm idempotent `rotate_if_due()`; tray ở v1, Task Scheduler là chế độ không resident | Khá | Task Scheduler bị policy chặn |

## Tauri thắng ở installer 1 MB, không thắng ở RAM

Phép so sánh gần nhất với "cùng app, cùng máy" là benchmark Dawloom tháng 10/2026: private memory cộng cả cây process, tính cả WebView2, trên một laptop i7-1185G7 ([Dawloom](https://dawloom.com/blog/tauri-vs-electron-vs-wpf-benchmark/)). Tác giả là agency bán cả Tauri lẫn Electron. Electron chỉ chạy một lần, và họ không công bố startup vì CPU lúc đo ở 100%.

| Build | Phân phối | Private memory khi mở cửa sổ | Trong tray |
|---|---|---|---|
| Tauri 2.12 | **1,05 MB** NSIS | 142 MB | **0 process WebView2** nếu destroy ([BettrFTHR PR 8](https://github.com/H4ck3rPr0G4m3r/BettrFTHR/pull/8)) [Tự báo cáo] |
| Electron 44 | 106,1 MB NSIS | 149 MB | Không có số |
| WPF .NET 10 | 0,07 MB zip (cần runtime) hoặc 56–60 MB zip self-contained | **74 MB** (framework-dependent) | 16,7 MB ([benchbar PR 70](https://github.com/askysh/benchbar/pull/70)) [Tự báo cáo] |
| Slint 1.19, software renderer | Không có số | 10,5 MB, chỉ là cửa sổ settings ([claude-rpc PR 7](https://github.com/inerthel-agi/claude-rpc/pull/7)) [Tự báo cáo] | **5,8 MB** (cùng nguồn) |
| Wails 2.16 | 4,5 MB zip | 205–209 MB | Không có số |

Các dòng không cùng phương pháp: chỉ cột private memory của Tauri, Electron, WPF và Wails đến từ cùng một nguồn. Chọn metric cũng đủ đổi kết luận. Một app Tauri mặc định đo được 72–75 MB private working set nhưng 141,8 MB private commit ([tauri#16618](https://github.com/tauri-apps/tauri/issues/16618)) [Tự báo cáo], nên đừng so số của các blog không ghi rõ cách đếm.

**Tốc độ khởi động là điểm yếu nhất của Tauri.** Benchmark CI của Elanis cho Tauri khoảng 650 ms và Electron khoảng 195 ms, kèm cảnh báo sai số lớn ([Elanis](https://github.com/Elanis/web-to-desktop-framework-comparison)). Một dự án khác đo trên laptop thật khoảng 1,3 s tới first paint, trong đó ~1,1 s là WebView2 ([Radiologytutor PR 148](https://github.com/drskatyal/Radiologytutor/pull/148)) [Tự báo cáo]. Code của app chỉ chiếm 29 ms trong lần đo CI của dự án đó, nên tối ưu Rust hay JS gần như không giúp. Native splash và tray menu native mới là thứ che được độ trễ này.

### Hide không giải phóng gì, destroy đưa WebView2 về 0

`hide()` giữ nguyên 72–75 MB private working set. `TrySuspend` hạ còn 6,3 MB sau 25–30 giây nhưng commit vẫn ~142 MB, và Tauri 2.12.7 chưa expose API này ([tauri#16618](https://github.com/tauri-apps/tauri/issues/16618)) [Tự báo cáo]. Một dự án khác báo `TrySuspend` vẫn để lại ~458 MB working set ([rightkit#955](https://github.com/bogusyogi/rightkit/issues/955)). Hai số khác metric nên không loại trừ nhau, nhưng đủ để không đặt cược vào suspend.

Destroy cho kết quả rõ hơn: cây WebView2 từ ~204 MB về **0 MB, 0 process**, mở lại hai lần không phình thêm, và khởi động `--minimized` cũng 0 process ([BettrFTHR PR 8](https://github.com/H4ck3rPr0G4m3r/BettrFTHR/pull/8)) [Tự báo cáo]. Cách làm là `"create": false` trong `tauri.conf.json` ([Tauri config](https://v2.tauri.app/reference/config/)), dựng lại window khi cần, và `prevent_exit` khi window cuối đóng. Rust host ở các dự án khác khoảng 6–17 MB private ([footnote#516](https://github.com/bobcat88/footnote/issues/516), [Radiologytutor](https://github.com/drskatyal/Radiologytutor/pull/148)) [Tự báo cáo], đo khi window còn tồn tại hoặc đang ẩn. **Không nguồn nào đo tổng RAM của host sau destroy, cũng không nguồn nào đo thời gian mở lại.** Nếu host quá nặng, dwall là tiền lệ cho phương án tách daemon: exe 0,63 MB, ~2 MB private working set, UI là process riêng chỉ chạy khi mở ([dwall](https://github.com/dwall-rs/dwall)) [Tự báo cáo].

### Windows 10 nghiêng cán cân về phía WebView2

Windows 10 vẫn chiếm **25,13%** desktop Windows tháng 10/2026 ([StatCounter](https://gs.statcounter.com/os-version-market-share/windows/desktop/worldwide)), và ESU miễn phí cho người dùng cá nhân đã gia hạn tới 12/10/2027 ([BleepingComputer](https://bleepingcomputer.com/news/microsoft/microsoft-quietly-extends-free-windows-10-esu-support-to-october-2027)). WebView2 tiếp tục cập nhật trên Windows 10 22H2 tới ít nhất tháng 10/2028 ([Microsoft Edge](https://learn.microsoft.com/en-us/deployedge/microsoft-edge-supported-operating-systems)). Ngược lại, .NET 10/11 chỉ hỗ trợ chính thức Windows 10 bản LTSC và Enterprise ([Microsoft](https://learn.microsoft.com/en-us/dotnet/core/install/windows)). Về UI, Mica chỉ có từ build 22000 ([Microsoft](https://learn.microsoft.com/en-us/windows/apps/design/style/mica)), còn acrylic resize/drag kém trên Windows 10 1903+ ([tauri Effect](https://docs.rs/tauri/2.12.7/tauri/window/enum.Effect.html)). Vì vậy hãy thiết kế nền solid/gradient cho Windows 10 trước, Mica là lớp bonus cho Windows 11 [Đề xuất].

### Máy dev chưa build được Tauri theo đường chính thức

**[Máy dev]** có rustc 1.99.0, toolchain `stable-gnu` đang là default và `stable-msvc` đã cài, WebView2 158. Máy không có thư mục Visual Studio, không có Windows Kits, không có `cl.exe`, và không có .NET SDK. `link.exe` duy nhất trên PATH của Git Bash là lệnh coreutils, không phải linker MSVC. Trong khi đó Tauri nói rõ chỉ hỗ trợ chính thức target MSVC ([Tauri installer](https://v2.tauri.app/distribute/windows-installer/)) và yêu cầu C++ Build Tools với workload "Desktop development with C++" ([Tauri prerequisites](https://v2.tauri.app/start/prerequisites/)). Các bước [Đề xuất]: cài Build Tools (cần quyền admin và vài GB), pin `stable-msvc` bằng `rust-toolchain.toml` để không ảnh hưởng dự án khác, build từ PowerShell. Đổi stack gần như không né được việc cài thêm: Slint với Skia renderer cũng cần MSVC ([Slint](https://docs.slint.dev/latest/docs/slint/guide/backends-and-renderers/backends_and_renderers/)), WPF cần .NET SDK, còn Slint software renderer trên toolchain GNU thì chưa ai thử trên máy này.

Với frontend, Svelte 5 nặng 9,7 KB nén với first paint 61,8 ms, React 19 là 51,4 KB và 228,9 ms ([js-framework-benchmark](https://krausest.github.io/js-framework-benchmark/2026/chrome158.html), đo trên Mac M4, median tính lại từ dữ liệu thô). Chênh lệch này đáng kể vì destroy-on-hide khiến JS được parse lại mỗi lần mở. Grid nên dùng `virtua` (~3 KB, hỗ trợ Svelte 5, [README](https://github.com/inokawa/virtua)), ô có kích thước biết trước từ metadata, và thumbnail đi qua custom async protocol từ Rust chứ không gửi bytes qua IPC [Đề xuất].

Về phát hành, NSIS cài per-user không cần admin và `downloadBootstrapper` thêm 0 MB, trong khi `offlineInstaller` thêm ~127 MB ([Tauri](https://v2.tauri.app/distribute/windows-installer/)). Không còn loại certificate nào mua được niềm tin SmartScreen tức thì. Azure Artifact Signing cho cá nhân chỉ mở ở Mỹ và Canada, OV khoảng $150–300/năm, SignPath miễn phí cho dự án open source đủ điều kiện ([Microsoft](https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/code-signing-options)). Quyết định có mở mã nguồn hay không vì thế ảnh hưởng thẳng tới chi phí ký app.

## Nguồn mới chỉ rẻ khi host giữ hết việc khó

### Wallhaven đủ tốt cho v1, với bốn cái bẫy

Docs công bố giới hạn **45 call/phút**, 24 kết quả mỗi trang, filter `atleast`, `resolutions`, `ratios` nhiều giá trị và 29 màu, không cần key cho SFW ([Wallhaven API](https://wallhaven.cc/help/api)). Một trang random cho 24 ứng viên, nên mỗi ngày chỉ cần khoảng một call cho mỗi dạng màn hình [Đề xuất].

Bẫy thứ nhất là `seed`. **Docs nói** random sinh ra seed có thể truyền giữa các trang để không lặp ([docs](https://wallhaven.cc/help/api)). **Test trực tiếp cho kết quả ngược lại**: gửi `seed=eM0dN4` bốn lần nhận về bốn seed khác nhau, lặp lại trang 1 trùng 0/24 id, và trang 2 đã lặp một id của trang 1 ([request](https://wallhaven.cc/api/v1/search?sorting=random&purity=100&ratios=21x9,32x9,9x16&seed=eM0dN4)) [Test 25/11/2026, không key]. Hãy tin test: thiết kế như thể `seed` không tồn tại, dedupe theo `id` ở client, và chỉ kiểm tra lại với API key như một việc phụ.

Bẫy thứ hai là purity. Không có key, `purity=111` vẫn trả về Sketchy và âm thầm bỏ NSFW ([request](https://wallhaven.cc/api/v1/search?sorting=random&purity=111&atleast=3840x2160)) [Test 25/11/2026]. App phải luôn gửi `purity=100` **và** kiểm tra `item.purity == "sfw"` trên từng item. Bẫy thứ ba là latency: bốn lần gọi random mất 0,6; 3,9; 9,9 và 4,5 giây ([API](https://wallhaven.cc/api/v1/search)) [Test 25/11/2026], nên timeout cần 20–30 giây và nút "ảnh tiếp theo" phải lấy từ hàng đợi đã tải sẵn [Đề xuất]. Bẫy thứ tư là pháp lý. ToS cập nhật lần cuối 3/8/2016, không có điều khoản nào về API hay app bên thứ ba, và footer ghi ảnh thuộc chủ sở hữu gốc ([ToS](https://wallhaven.cc/terms)). FAQ đề nghị không tải hàng loạt ([FAQ](https://wallhaven.cc/faq)). Hệ quả [Đề xuất]: thiết bị người dùng tải thẳng từ Wallhaven, không mirror, không bán quyền xem ảnh, ghi "uploaded by" thay vì "author" và luôn có link trang gốc. Nếu định thu tiền, hãy email hỏi chủ site trước.

### Một trait nhỏ, capability khai báo, host làm phần còn lại

Jackett là bằng chứng mạnh nhất cho hướng declarative: **589 định nghĩa YAML so với 86 file C#** ([Jackett](https://github.com/Jackett/Jackett)), tức khoảng 87% nguồn chỉ là dữ liệu. Mihon cho thấy một tập filter đóng (Select, Text, CheckBox, Sort…) đủ để UI render bộ lọc của mọi nguồn mà không viết UI riêng ([Filter.kt](https://github.com/mihonapp/mihon/blob/main/source-api/src/main/kotlin/eu/kanade/tachiyomi/source/model/Filter.kt)). Muzei tách `token`, `persistentUri` và `webUri` ([Artwork.kt](https://github.com/muzei/muzei/blob/main/muzei-api/src/main/java/com/google/android/apps/muzei/api/provider/Artwork.kt)), đúng thứ cần để chống lặp và ghi nguồn. Ghép lại thành thiết kế sau [Đề xuất, chưa compile]:

```rust
#[async_trait]
trait Provider: Send + Sync {
    fn info(&self) -> &ProviderInfo;                                   // id, api_version, auth, caps, rate, cache, legal
    async fn list(&self, req: ListRequest, cx: &Ctx) -> Result<Page, ProviderError>;  // bắt buộc duy nhất
    async fn resolve(&self, it: &Item, target: Size, cx: &Ctx) -> Result<ImageRef, ProviderError>; // default: it.full
    async fn on_event(&self, ev: ItemEvent, cx: &Ctx) {}               // default: no-op
}
struct Caps { modes, pagination, resolution, aspect, colors, custom: Vec<FilterSpec> }
struct Page { items: Vec<Item>, next: Option<Cursor>, total: Option<u64> }  // Cursor là chuỗi opaque do provider sở hữu
enum ProviderError { RateLimited{retry_after}, Auth, NotFound, Empty, Blocked, Network, BadResponse, Unsupported }
```

Host giữ HTTP client, User-Agent, token bucket theo provider, retry, disk cache, secret, decode và safety filter. Filter nào provider không hỗ trợ thì host lọc local với số trang thêm có giới hạn. Lớp lọc local này bắt buộc, vì filter phía server lỏng: Openverse `size=large` trả về ảnh chỉ 1792x1146 ([Openverse](https://api.openverse.org/v1/images/?q=mountain&aspect_ratio=wide&size=large&page_size=1&license_type=commercial&format=json)) [Test 25/11/2026]. `Cursor` opaque che được page (Wallhaven), offset (Met) và `continue` (Commons). `resolve()` cần cho nguồn chỉ trả id hoặc URL hết hạn.

| Cơ chế thêm nguồn | Công sức ước lượng [Đề xuất] | Cần phát hành app? | Khi nào dùng |
|---|---|---|---|
| Trait compile-in | 100–300 dòng + test | Có | v1, API lạ, thư mục local |
| Declarative manifest (TOML + JSONPath) | 40–100 dòng | Không | Nguồn REST/JSON thuần, từ nguồn thứ hai cùng dạng Wallhaven |
| Script nhúng (Rhai) | 50–150 dòng | Không | Chỉ khi cần ký request |
| WASM (Extism) | Toolchain riêng | Không | Plugin không tin cậy, chưa cần |

Đừng viết manifest engine ngay. Với một nguồn REST, bạn sẽ thiết kế ngôn ngữ manifest vừa khít Wallhaven. Wallhaven và thư mục local là hai hình dạng đủ khác nhau để thử trait. Wikimedia (cursor `continue`, attribution dạng HTML, User-Agent bắt buộc) nên viết native như bài stress test. Openverse, một API REST/JSON thuần, mới là lúc biết manifest cần những phép biến đổi nào [Đề xuất].

### Nguồn kế tiếp: Wikimedia trước, Unsplash không bao giờ

| Nguồn | Ràng buộc quyết định | Kết luận |
|---|---|---|
| Thư mục local | Không có | v1 |
| Wikimedia Commons | 200 req/phút nếu User-Agent có liên hệ, 10 nếu không ([Wikimedia](https://www.mediawiki.org/wiki/Wikimedia_APIs/Rate_limits)) | Nguồn online thứ hai, viết native |
| Openverse | 20/phút, **200/ngày** khi ẩn danh ([API](https://api.openverse.org/v1/images/?q=mountain&aspect_ratio=wide&size=large&page_size=1&license_type=commercial&format=json)) [Test 25/11/2026] | Nguồn thứ ba, khởi đầu cho manifest |
| The Met | CC0, không key, search chỉ trả id nên tốn N+1 call, search cũ tắt từ 1/10/2026 ([Met](https://metmuseum.github.io/)) | Ngách |
| NASA APOD | Endpoint chuyển sang backend mới ngày 1/12/2026, có cả video ([apod-api](https://github.com/nasa/apod-api)) | Chờ sau khi chuyển |
| Bing, Spotlight | Không có API công khai; nguồn Bing của KDE từng hỏng 1/2025 ([KDE](https://bugs.kde.org/show_bug.cgi?id=498655)) | Tùy chọn, tắt mặc định |
| Pixabay | Chỉ 1280 px nếu chưa được duyệt, 100 req/60 s **theo key** ([Pixabay](https://pixabay.com/api/docs/)) | Rủi ro, bỏ qua |
| Unsplash | "You cannot replicate… wallpaper applications" ([Unsplash](https://help.unsplash.com/en/articles/2511245-unsplash-api-guidelines)) | **Không làm** |
| Pexels | Cấm "making Pexels content available as a wallpaper app" ([Pexels](https://www.pexels.com/api/documentation/)) | **Không làm** |
| Reddit | Public API đóng tháng 3/2027 ([TechCrunch](https://techcrunch.com/2026/09/30/reddit-is-killing-rss-feeds-ending-public-api-access-because-of-ai-bots/)) | **Không làm** |

Key nhúng trong binary là key công khai; Variety chỉ XOR key Unsplash của mình ([Variety](https://github.com/varietywalls/variety/blob/master/variety/plugins/builtin/downloaders/UnsplashDownloader.py)). v1 vì thế chỉ dùng nguồn không key. Sau này key của người dùng nằm trong Credential Manager/DPAPI và đi qua header `X-API-Key`, không nằm trong URL [Đề xuất].

## Mỗi màn hình cần một query và một ảnh riêng

**Lấy danh sách màn hình từ chính `IDesktopWallpaper`.** API này chạy unpackaged từ Windows 8 và set theo từng monitor ([Microsoft](https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nn-shobjidl_core-idesktopwallpaper)). `EnumDisplaySettingsW` luôn trả physical pixels nên hợp để đối chiếu ([Microsoft](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-enumdisplaysettingsw)). Có ba bẫy. Enumeration gồm cả monitor đã rút, khi đó `GetMonitorRECT` trả `S_FALSE` ([Microsoft](https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nf-shobjidl_core-idesktopwallpaper-getmonitorrect)), mà Rust lại map `S_FALSE` thành `Ok` nên phải kiểm tra RECT rỗng (chưa test). tao chỉ bật Per-Monitor-V2 lúc runtime ([tao dpi.rs](https://github.com/tauri-apps/tao/blob/dev/src/platform_impl/windows/dpi.rs)) còn manifest mặc định không khai báo ([manifest](https://github.com/tauri-apps/tauri/blob/dev/crates/tauri-build/src/windows-app-manifest.xml)), nên chế độ headless cần manifest riêng. Cuối cùng, đừng dùng `screen.width * devicePixelRatio` trong WebView, vì nó làm tròn và chỉ thấy màn hình hiện tại [Đề xuất].

### Query theo bucket, chấm điểm lại ở local

`ratios=21x9,32x9,9x16` hoạt động nhưng khớp theo bucket: `21x9` gồm cả 2,33, 2,37 và 2,39 ([request](https://wallhaven.cc/api/v1/search?sorting=random&purity=100&ratios=21x9,32x9,9x16)) [Test 25/11/2026]. Keyword `portrait` không có trong docs và trả về ratio từ 0,37 đến 0,86 ([request](https://wallhaven.cc/api/v1/search?sorting=random&purity=100&ratios=portrait&atleast=1080x1920)) [Test 25/11/2026], nên hãy liệt kê ratio cụ thể.

| Màn hình (physical) | Query đầu [Đề xuất] | Nới lần một [Đề xuất] |
|---|---|---|
| 2560x1440 | `atleast=2560x1440&ratios=16x9` | thêm `16x10` (mất 10%) |
| 2880x1920 (3:2) | `atleast=2880x1920&ratios=3x2` | thêm `16x10` (mất 6%) |
| 3440x1440 | `atleast=3440x1440&ratios=21x9` | thêm `16x9` (mất 25,6%) |
| 5120x1440 | `atleast=5120x1440&ratios=32x9` | thêm `21x9` (mất ~33%), chỉ khi người dùng cho phép |
| 1440x2560 (dọc) | `atleast=1440x2560&ratios=9x16` | thêm `10x16`, `9x18` (mất 10–11%) |

Với monitor `Mw x Mh` (ratio `Rm`) và ảnh `Iw x Ih` (ratio `Ri`), hai số đủ để xếp hạng: hệ số phóng `s = max(Mw/Iw, Mh/Ih)` và tỉ lệ mất do crop `L = 1 - min(Rm/Ri, Ri/Rm)`. Chính sách [Đề xuất]: chế độ tự động bắt đầu với `s <= 1` và `L <= 0,12`. Khi pool cạn mới nới tới `L <= 0,30`, rồi `s <= 1,15`. Vẫn cạn thì giữ ảnh hiện tại. Trong ảnh đạt chuẩn, ưu tiên ảnh nhỏ nhất đủ dùng để tiết kiệm băng thông. Ảnh 16:9 trên màn dọc mất 68,4%, nên **màn dọc phải có query riêng**. Không có nghiên cứu nào về ngưỡng crop người dùng chấp nhận được, các ngưỡng trên là mặc định để tinh chỉnh.

### Ảnh PNG đúng pixel né được ba vấn đề của Explorer

Position mode là giá trị toàn cục, không theo monitor ([Microsoft](https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nn-shobjidl_core-idesktopwallpaper)). Nếu app tự crop và resize về đúng kích thước rồi đặt `FILL`, mode này không còn quan trọng [Đề xuất]. Explorer cũng re-encode JPEG vào `TranscodedWallpaper`, mặc định chất lượng 85% theo nguồn cộng đồng ([Winhelponline](https://www.winhelponline.com/blog/increase-desktop-wallpaper-jpeg-image-quality-windows-10/)). Trên **[Máy dev]** (26H2), một JPEG 3840x2400 nặng 1,60 MB thành 0,47 MB, cùng kích thước pixel. PNG được báo là không bị nén lại (cùng nguồn), nên hãy xuất PNG với tên file mới theo content hash và không chỉnh registry của người dùng [Đề xuất]. App cũng tự decode WebP, vì hỗ trợ WebP làm desktop background chỉ mới vào Windows 11 qua KB5077241 tháng 2/2026 ([Microsoft](https://support.microsoft.com/kb/5077241)).

Không dùng `SystemParametersInfo` làm đường chính: nó không set theo monitor, và một phép đo trên một máy cho thấy caller bị chặn 1–5 giây, so với ~1 ms của `IDesktopWallpaper` ([BingWall #23](https://github.com/mrvonwyl/BingWall/issues/23)) [Tự báo cáo]. Cùng phép đo đó thấy Explorer treo 0,9–1,6 giây mỗi lần đổi nếu bật accent color tự động, và chỉ ~50–60 ms nếu tắt. Đó là lý do tốt để không xoay ảnh liên tục. Nguồn cộng đồng mâu thuẫn về apartment STA hay MTA ([deskmakeover](https://github.com/nicepkg/deskmakeover/blob/HEAD/crates/dm-windows/src/com/apartment.rs), [BingWall #24](https://github.com/mrvonwyl/BingWall/issues/24)). Hãy đặt mọi lời gọi trên một COM thread riêng, dùng `CLSCTX_ALL`, rồi để prototype chốt.

**Giới hạn phải nói thẳng với người dùng: virtual desktop.** Trên Windows 11, các API công khai chỉ đổi chắc chắn desktop hiện tại; đường duy nhất tới mọi desktop là một interface nội bộ đã đổi IID/vtable bốn lần từ 21H2 ([BingWall research](https://github.com/mrvonwyl/BingWall/blob/research/virtual-desktops/research/virtual-desktops.md)). Có một báo cáo ngược lại rằng SPI đổi tất cả ([VirtualDesktopAccessor #85](https://github.com/Ciantic/VirtualDesktopAccessor/issues/85)). v1 chỉ hỗ trợ desktop hiện tại [Đề xuất].

## Daily random là hàm idempotent, không phải timer

Lỗi đắt nhất của app cùng loại không phải random kém mà là **lịch chết âm thầm**. Client Wallhaven nhiều sao nhất có hai issue (#38, #39) về rotation tự dừng ([leoFitz issues](https://github.com/leoFitz1024/wallhaven/issues?q=is%3Aissue)), còn người dùng Dynamic Theme than "stuck for 24 hours" khi không thích ảnh ([Store reviews](https://apps.microsoft.com/api/products/getReviews/9NBLGGH1ZBKW?gl=US&hl=en-us&pgNo=1&noItems=25&orderBy=3)). Cách chống là một hàm duy nhất `rotate_if_due()` [Đề xuất]: tính ngày local theo giờ đổi do người dùng chọn, thoát ngay nếu `last_applied_date >= today`, kẹp về hôm nay nếu ngày đã lưu nằm ở tương lai. Khi đó timer, logon, unlock và resume đều gọi được mà không đổi ảnh hai lần. Lỡ ba ngày chỉ đổi một lần, DST và đổi múi giờ tự triệt tiêu.

| Mô hình chạy | Chi phí nền | Điểm mạnh | Rủi ro |
|---|---|---|---|
| Tray Rust + timer, WebView bị hủy | Rust host (chưa đo) | Có next/skip/like tức thì | Phụ thuộc autostart |
| **Task Scheduler** chạy `app.exe --rotate` | 0 khi không chạy | Nhẹ nhất, không cần admin | Policy, dọn task khi gỡ, chưa test |
| Run key, kiểm tra rồi thoát | 0 sau logon | Đơn giản | Bỏ lỡ máy bật nhiều ngày |

Khuyến nghị [Đề xuất]: **v1 dùng tray**, vì next, skip và "never again" đòi một process thường trú. Viết sẵn chế độ `--rotate` gọi cùng hàm đó, rồi cung cấp Task Scheduler cho người dùng tắt tray sau khi prototype xác nhận. Nếu làm, có ba chi tiết đã có nguồn. User thường chỉ được đăng ký logon trigger khi trigger chỉ định **chính user đó** ([MS-TSCH](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-tsch/d90eccec-83e9-450c-b144-17633c47be9a)). `DisallowStartIfOnBatteries` mặc định **True**, nên laptop chạy pin sẽ bỏ qua task ([Microsoft](https://learn.microsoft.com/en-us/windows/win32/taskschd/taskschedulerschema-disallowstartifonbatteries-settingstype-element)). `StartWhenAvailable` trễ mặc định 10 phút, và docs mâu thuẫn về điều kiện áp dụng ([API page](https://learn.microsoft.com/en-us/windows/win32/api/taskschd/nf-taskschd-itasksettings-get_startwhenavailable)). Với autostart, bản HEAD của plugin dùng `auto-launch` ở mode `Dynamic`, thử HKLM trước và ghi đè lựa chọn của người dùng trong Task Manager ([auto-launch](https://github.com/zzzgydi/auto-launch/blob/HEAD/src/windows.rs)). Hãy gọi `CurrentUser` trực tiếp và chỉ khi người dùng bật [Đề xuất].

### Random local thay cho random của server

Vì `seed` không đáng tin, random phải nằm ở client [Đề xuất]. Với thư mục và favorites, dùng shuffle bag Fisher–Yates có lưu cursor, và cấm N ảnh cuối xuất hiện ở đầu vòng mới. Với Wallhaven, giữ bảng `seen` theo `(provider, id)` và bỏ qua id đã gặp. Feed random của Wallhaven không đều về chất lượng, nên có một biến thể đáng thử: dùng sort ổn định như `toplist` với `topRange`, chọn trang ngẫu nhiên trong `1..last_page`, rồi shuffle local. Cách này vừa lọc chất lượng vừa không cần seed, nhưng độ ổn định thứ tự và kích thước pool chưa được kiểm tra; ví dụ `topRange=1w` chỉ có 467 ảnh trước khi lọc ratio ([API](https://wallhaven.cc/api/v1/search)) [Test 25/11/2026].

Tách **prefetch** (cần mạng, bỏ qua khi metered hoặc battery saver) khỏi **apply** (chỉ chạm file `ready` trên đĩa) [Đề xuất]. Sau mỗi lần đổi, chuẩn bị 3 ảnh cho mỗi nhóm monitor cùng kích thước: tải vào `.part`, decode, crop, resize, encode PNG, atomic rename. `fast_image_resize` xử lý ảnh 16 MP bằng Lanczos3 trong ~14 ms với AVX2 ([benchmark của dự án](https://github.com/cykooz/fast_image_resize/blob/main/benchmarks-x86_64.md), Ryzen 9 5950X, đích 852x567) [Tự báo cáo], nên resize nhiều khả năng không phải nút cổ chai. Hoãn đổi khi `SHQueryUserNotificationState` báo busy, full-screen D3D hoặc presentation ([Microsoft](https://learn.microsoft.com/en-us/windows/win32/api/shellapi/ne-shellapi-query_user_notification_state)). Giữ ảnh đang dùng và 10 ảnh trước cho undo, và lưu wallpaper gốc của người dùng trước lần đổi đầu tiên.

## v1 hẹp, và mười một câu hỏi phải trả lời trước

Thị trường vừa mở một cửa: tháng 9/2026 Bing Wallpaper đặt quảng cáo phim toàn desktop ([Windows Latest](https://www.windowslatest.com/2026/09/08/microsoft-said-it-would-cut-windows-11-ads-then-put-a-harry-potter-ad-on-your-desktop-if-you-use-bing-wallpaper/)). Nhưng "Tauri và nhẹ" chưa kéo được người dùng: [Wallery](https://github.com/dmitrykirsh/wallery) (Tauri, 2,3 MB) có 21 lượt tải và [WallFlow](https://github.com/Toastyfil/WallFlow) (Tauri, 4,6 MB) có 2, theo GitHub API ngày 25/11/2026. WallFlow còn xin cố định 1920x1080+ thay vì theo màn hình, còn WallGaze thu phí auto-rotation ([Store](https://storeedgefd.dsx.mp.microsoft.com/v9.0/products/9NJLDD2934RK?market=US&locale=en-us&deviceFamily=Windows.Desktop)). Khoảng trống thật là **ghép đúng resolution theo từng màn hình với daily random miễn phí và đáng tin**, gói trong UI gọn. Ưu tiên dưới đây là tổng hợp từ mẫu review và issue nhỏ, không phải khảo sát.

| v1 | Sau v1 | Không làm |
|---|---|---|
| Wallhaven SFW + thư mục local sau `Provider` trait | Manifest engine, Wikimedia, Openverse | Unsplash, Pexels, Reddit, scraping |
| Tự detect monitor, query và crop theo từng màn | Smart crop, khung mờ cho ratio lạ | Đổi `JPEGImportQuality` của người dùng |
| Daily random, skip, previous, undo, catch-up, offline queue | Nhiều interval, hotkey | Paywall daily change, AI feed |
| Tray: next, previous, like, never again, pause, open | Task Scheduler mode nếu prototype chưa đạt | Autostart không hỏi |
| Grid ảo hóa, search, filter, preview, set một click | Wallhaven API key, collections | NSFW trong chế độ tự động |
| Favorites, history, link nguồn và uploader | Lock screen, dim/blur | Widget, video, thời tiết |
| NSIS per-user, updater, không telemetry | Store MSIX đóng gói thủ công | Interface virtual desktop nội bộ |

Ngưỡng đạt trong bảng sau là [Đề xuất]. P0 là điều kiện chạy; P1–P3 quyết định stack; P4–P8 quyết định thiết kế; P9–P10 quyết định phát hành.

| # | Câu hỏi | Cách thử | Đạt khi | Nếu trượt |
|---|---|---|---|---|
| P0 | Build Tauri MSVC được trên máy này? | Cài Build Tools, build app mặc định | Ra NSIS installer | Dừng, xét lại stack |
| P1 | Mở lại sau destroy mất bao lâu? | Đo click tray tới first paint, 20 lần, một máy Win10 và một máy Win11 yếu | p95 < 1 s | Giữ WebView 5–10 phút rồi mới hủy; nếu cần <150 ms thì Slint hoặc WPF |
| P2 | Host sau destroy nặng bao nhiêu? | Private bytes và private WS cả cây, sau 5 phút | < 15 MB private | Tách daemon kiểu dwall |
| P3 | Grid 1.000 thumbnail có mượt? | Cùng dữ liệu trên Tauri và Slint, đo frame time và RAM | p95 < 16,7 ms, không giật | So Slint GPU renderer và RAM của nó |
| P4 | `IDesktopWallpaper` chạy đúng đa màn hình? | STA và MTA; 100% + 150%; rút màn; màn dọc; `--rotate` headless | Đúng pixel, bỏ qua màn đã rút | Dùng vtable để đọc HRESULT thô |
| P5 | PNG có giữ chất lượng, dung lượng ra sao? | So `TranscodedWallpaper` với nguồn, ảnh 4K thật | Không mất chi tiết, PNG < 20 MB | JPEG q95–100 cho ảnh lớn |
| P6 | Pool Wallhaven có đủ cho 21:9, 32:9, dọc? | Đếm `total` theo query, SFW + General, sau lọc local | ≥ 365 ảnh mỗi dạng | Nới L hoặc thêm nguồn |
| P7 | `toplist` + trang ngẫu nhiên có ổn định? `seed` có chạy với key? 429 trả header gì? | Gọi tiết chế, dưới 25/phút; không cố tình ép 429 | Thứ tự ổn định trong phiên | Random trang 1 + `seen` |
| P8 | Trait có thật sự rẻ? | Viết Wikimedia adapter native | Không sửa UI, < 300 dòng | Sửa `Caps` và `Cursor` trước v1 |
| P9 | Task Scheduler và autostart trên user thường? | Standard account, pin, Modern Standby, path có dấu cách | Đổi đúng một lần mỗi ngày | Chỉ giữ tray |
| P10 | Ký app thế nào? | Quyết open source hay không; hỏi SignPath | Có đường ký cụ thể | OV hoặc chấp nhận cảnh báo SmartScreen |

Virtual desktop nằm ngoài bảng vì v1 đã loại nó, nhưng hãy thử một lần trên 26H2 khi chạy P4 để viết đúng câu cảnh báo trong app. Hai câu hỏi không prototype nào trả lời được là Wallhaven có chấp nhận app thương mại không, và nhu cầu cho một client Wallhaven lớn tới đâu. Câu đầu cần email, câu sau cần beta nhỏ.

## Kết luận

Câu hỏi ban đầu "Tauri có nhẹ không" hóa ra đặt sai chỗ. Với app wallpaper, trạng thái chiếm gần trọn thời gian sống là cửa sổ đóng, và ở trạng thái đó kiến trúc process quyết định nhiều hơn framework. Một app Tauri chỉ `hide()` sẽ nặng như Electron cả ngày, còn một app Tauri không có WebView nào là một chương trình Rust nhỏ. Hệ quả thực tế là ranh giới code: scheduler, provider, pipeline ảnh và Win32 nên nằm trong một crate Rust không phụ thuộc Tauri ngay từ commit đầu. Ranh giới đó làm cho chế độ `--rotate`, việc tách daemon và cả việc đổi sang Slint đều trở thành thay vỏ chứ không phải viết lại, nên nó cũng là bảo hiểm rẻ nhất nếu P1 đến P3 trượt.

Điểm thứ hai là docs của nguồn ảnh không phải hợp đồng. `seed` sai với docs, và chỉ riêng giai đoạn 2025–2026 Reddit, Met, APOD và nguồn Bing đều thay đổi. "Thêm nguồn nhẹ nhàng" vì thế phải đi cùng "mất nguồn nhẹ nhàng": hàng đợi ảnh trên đĩa, fallback sang nguồn khác và contract test với fixture ghi sẵn đáng làm trước mọi cơ chế plugin.
