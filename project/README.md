# Project

Cargo workspace chứa toàn bộ source code. Giấy phép: GPL-3.0-only (xem `LICENSE` ở gốc repo).

- `apps/desktop`: app Windows, giao diện Slint (software renderer)
- `packages/core`: kiểu dữ liệu, trait `Provider`, tính độ vừa màn hình. Không có UI, mạng hay API hệ điều hành
- `packages/host`: HTTP, rate limit, cache đĩa, giải mã và cắt ảnh
- `packages/provider-wallhaven`: nguồn ảnh Wallhaven (chỉ SFW, không cần key)
- `packages/platform-windows`: đọc độ phân giải màn hình, mở link

Cần Visual Studio Build Tools với workload "Desktop development with C++". `rust-toolchain.toml` tự chọn toolchain MSVC cho riêng folder này.

Chạy các lệnh từ folder này:

```sh
cargo run --release -p wallpaper-desktop   # chạy app
cargo test --workspace                     # chạy test
cargo clippy --workspace --all-targets     # kiểm tra lỗi thường gặp
```

Công cụ render ngoài màn hình (dùng dữ liệu thật, không hiện cửa sổ):

```sh
cargo run --release --example offscreen -- shots <folder>       # chụp giao diện
cargo run --release --example offscreen -- bench 3840 2100 1.5  # đo thời gian vẽ
```

## Thêm nguồn ảnh

1. Tạo crate mới trong `packages/`, viết một kiểu hiện thực `wallpaper_core::Provider`.
2. Khai báo `Capabilities`: kiểu sắp xếp, danh mục, tỉ lệ, màu. Giao diện tự dựng bộ lọc từ đó.
3. Trong `list`, đổi `ListRequest` thành URL rồi đổi câu trả lời thành `Item`.

App hiện chỉ dùng một nguồn, truyền vào ở `apps/desktop/src/main.rs`. Chưa có phần chọn giữa nhiều nguồn.
