# Project

Cargo workspace chứa toàn bộ source code. Giấy phép: GPL-3.0-only (xem [LICENSE](../LICENSE)).

- `apps/desktop`: app Windows, giao diện Slint (software renderer)
- `packages/core`: kiểu dữ liệu, trait `Provider`, tính độ vừa màn hình. Không có UI, mạng hay API hệ điều hành
- `packages/host`: HTTP, rate limit, cache đĩa, giải mã và cắt ảnh, lịch sử hoàn tác hình nền
- `packages/provider-wallhaven`: nguồn ảnh Wallhaven (chỉ SFW, không cần key)
- `packages/platform-windows`: đọc màn hình và virtual desktop, đặt và trả lại hình nền, mở link. Chi tiết ở [docs/windows-wallpaper.md](../docs/windows-wallpaper.md)

Chỉ build được trên Windows. Cần Visual Studio Build Tools với workload "Desktop development with C++". `rust-toolchain.toml` tự chọn toolchain MSVC cho riêng folder này.

Chạy các lệnh từ folder này:

```sh
cargo run --release -p wallpaper-desktop   # chạy app
cargo test --workspace                     # chạy test
cargo clippy --workspace --all-targets     # kiểm tra lỗi thường gặp
```

Test mặc định không đổi hình nền. Test trên desktop thật phải bật riêng, và tự trả lại hình nền cũ:

```powershell
$env:WALLPAPER_TEST_PICTURE = "C:\đường\dẫn\ảnh.png"
cargo test -p wallpaper-platform-windows -- --ignored round_trip
```

Công cụ render ngoài màn hình (dùng dữ liệu thật, không hiện cửa sổ):

```sh
cargo run --release --example offscreen -- shots <folder>       # chụp giao diện
cargo run --release --example offscreen -- bench 3840 2100 1.5  # đo thời gian vẽ
cargo run --release --example offscreen -- wallpaper 10         # đổi hình nền thật 10 giây rồi hoàn tác
```

Dữ liệu của app nằm ở `%LOCALAPPDATA%\WallpaperDesktop`: `thumbs` và `originals` là cache, `pictures` chứa hình nền đang dùng và lịch sử hoàn tác. Xóa `pictures` sẽ mất lịch sử hoàn tác và có thể mất ảnh đang hiện.

## Thêm nguồn ảnh

1. Tạo crate mới trong `packages/`, viết một kiểu hiện thực `wallpaper_core::Provider`.
2. Khai báo `Capabilities`: kiểu sắp xếp, danh mục, tỉ lệ, màu. Giao diện tự dựng bộ lọc từ đó.
3. Trong `list`, đổi `ListRequest` thành URL rồi đổi câu trả lời thành `Item`.

App hiện chỉ dùng một nguồn, truyền vào ở `apps/desktop/src/main.rs`. Chưa có phần chọn giữa nhiều nguồn.
