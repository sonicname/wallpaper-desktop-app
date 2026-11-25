# Slint vs Tauri: kết quả đo và prototype

**Quyết định (25/11/2026): dùng Slint 1.19 với software renderer.** Tauri là phương án quay lại nếu giao diện dựng bằng Slint không đủ đẹp. Lõi Rust (provider, lịch, xử lý ảnh, Win32) phải nằm trong crate không phụ thuộc UI để đổi được vỏ.

Máy đo: Windows 11 26H2 (build 26300), RTX 4060 Laptop, 16 luồng, RAM 15 GB, màn 2560×1440 144 Hz, scale 100%, cắm sạc. rustc 1.99.0 MSVC, WebView2 158, Slint 1.19.1, Tauri 2.12.

## Cửa sổ 1200×800, 1.000 ảnh tải lười, 4 cột

Khởi động, mở lại và RAM lấy từ 2 vòng đo, thứ tự đảo ngược, không có người dùng máy (trừ đầu lần chạy đầu tiên). Dữ liệu: [raw/window-2-rounds.txt](raw/window-2-rounds.txt).

| | Slint software | Tauri 2.12 | Slint FemtoVG | Slint Skia |
|---|---|---|---|---|
| Installer (NSIS) | 4,2 MB | 1,2 MB | 4,2 MB | 6,8 MB |
| Khởi động tới lúc thấy ảnh (trung vị mỗi vòng) | 113–126 ms | 507–521 ms | 269–280 ms | 494–514 ms |
| Mở lại sau khi hủy cửa sổ (trung vị mỗi vòng) | 97–98 ms | 383–420 ms | 161–174 ms | 559–567 ms |
| RAM private khi mở | 12 MB | 170 MB | 117–120 MB | 226–227 MB |
| RAM private khi cuộn | 33 MB | 291–294 MB | 150–153 MB | 249–251 MB |
| RAM private trong tray, sau khi hủy cửa sổ | 3–5 MB | 5–6 MB | 73–81 MB | 11–15 MB |
| Khung hình thật khi cuộn (1 lần, [raw](raw/present-fps.txt)) | 143 fps | 144 fps | 137 fps | 144 fps |

## Phóng to 2560×1369, 8 cột

Slint software và Tauri đo 2 lần, hai bản còn lại 1 lần. Dữ liệu: [raw/present-fps.txt](raw/present-fps.txt).

| | Slint software | Tauri | Slint FemtoVG | Slint Skia |
|---|---|---|---|---|
| Khung hình thật khi cuộn | 134–135 fps | 144 fps | 124 fps | 144 fps |
| RAM private khi cuộn | 65 MB | 428–431 MB | 248 MB | 272 MB |
| CPU khi cuộn (% một nhân) | 101–105% | 79–93% | 49% | 39% (máy đang bận) |

## Cách đo

- **Khởi động / mở lại:** harness bên ngoài dò pixel của ảnh đầu tiên (màu magenta) trên màn hình, giống hệt cho mọi bản. Sai số khoảng một tick của timer Windows (tới ~16 ms).
- **RAM / CPU:** cộng cả cây process, gồm `msedgewebview2.exe`. RAM là private bytes.
- **Khung hình thật:** đếm số lần một dải pixel trong cửa sổ test thay đổi, chỉ hash trong bộ nhớ.

Dòng `frames` trong `window-2-rounds.txt` đếm callback animation trong app, không phải khung hình hiển thị. Đừng dùng nó để so fps.

## Giới hạn

- Một máy mạnh, chỉ Windows 11. Chưa đo Windows 10, máy yếu, 4K, hoặc lần mở đầu sau khi khởi động lại máy.
- Không tính bộ nhớ GPU. Private bytes của bản software không gồm framebuffer; tổng working set khi mở là 25–33 MB qua hai lần đo (phóng to: 45 MB).
- Bản Tauri dùng JavaScript thuần, nên là mức nhẹ nhất của Tauri, chưa phải Svelte. Cache ảnh của Chromium không bị giới hạn.
- Số cửa sổ thường đo trước khi thêm chế độ phóng to (danh sách 250 hàng, chưa có PNG); mã lưu ở đây là bản cuối (125 hàng).
- Installer Slint chỉ chứa exe, chưa có uninstaller hay updater.
- Lần đo phóng to đầu tiên của Slint bị loại vì cửa sổ không thực sự phóng to.
- Đo với 24 thumbnail Wallhaven nhân bản. Ảnh đó không lưu ở đây vì bản quyền thuộc tác giả; `make-thumbs.ps1` tạo ảnh tổng hợp thay thế nên số đo lại có thể lệch.

## Lưu ý khi làm tiếp với Slint

- Software renderer không có kính mờ (`backdrop-blur`), xoay hay scale phần tử. Bo góc, đổ bóng và tiếng Việt có dấu đã thử và hiển thị đúng.
- Cuộn khi phóng to ở 1440p đã dùng hết một nhân CPU, nên cần đo 4K sớm. Skia là lối thoát nhưng tốn khoảng 270 MB.
- FemtoVG giữ 73–81 MB trong tray sau khi đóng cửa sổ, không hợp với app chạy nền.
- Giấy phép Slint: GPLv3, hoặc royalty-free có ghi công (`AboutSlint` hoặc badge). Cần chọn trước khi phát hành.
- Build cần toolchain MSVC. Toolchain GNU mặc định trên máy thiếu assembler và dừng ở lỗi `dlltool: CreateProcess`.
- `set_maximized(true)` ngay sau `show()` không có tác dụng trong phép thử này; gọi trễ 150 ms thì được.

## Chạy lại

```powershell
./make-thumbs.ps1                    # tạo 1.000 ảnh test trong thumbs/
./run.ps1 -Only sw -Mode present     # đo nhanh một bản (~20 giây)
./run.ps1                            # đo đủ 4 bản (~13 phút)
./run.ps1 -Mode present -Maximized   # đo khung hình khi phóng to
```

Cửa sổ `bench-window` sẽ tự mở và nằm trên cùng trong lúc đo. Bản Tauri cần WebView2.
