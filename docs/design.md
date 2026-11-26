# Giao diện

Code: `project/apps/desktop/ui/`. Bản làm lại ngày 26/11/2026.

## Bố cục

- **Thanh điều hướng bên trái** (`app.slint`): mỗi nguồn ảnh một mục, có icon. Mục "Mỗi ngày" nằm dưới cùng, có chấm màu khi đang bật.
- **Phần đầu**: ô tìm kiếm (nguồn không tìm kiếm được thì hiện tên nguồn), kiểu sắp xếp dạng nút gộp, số ảnh. Hàng dưới là bộ lọc và nút hoàn tác.
- **Lưới** (`grid.slint`): rê chuột lên ảnh hiện độ phân giải, nút trái tim và nút "Đặt làm nền". Bấm vào ảnh mở màn xem trước.
- **Xem trước** (`detail.slint`): phía sau là bản mờ, tối của chính ảnh đó. Ảnh nằm trong khung màn hình, cắt đúng như khi đặt thật. Thông tin và nút nằm trong một thẻ nổi bên phải.
- **Mỗi ngày** (`daily.slint`): thẻ nổi bên phải, cài đặt gom theo nhóm.

## Màu và kích thước

Tất cả nằm trong `theme.slint`. Màu nhấn duy nhất là vàng hổ phách, chỉ dùng cho nút chính, trạng thái đang chọn và trái tim. Màu xanh lá và đỏ chỉ dùng để báo ảnh vừa hay không vừa màn hình.

Hai giá trị phải khớp với `project/apps/desktop/src/grid.rs`, vì thumbnail được bo góc sẵn trong Rust:

| `theme.slint` | `grid.rs` |
|---|---|
| `background` | `BACKGROUND` |
| `radius` | `RADIUS` |
| `line` | `LINE` (viền cửa sổ) |

## Ràng buộc của renderer

App vẽ bằng CPU (software renderer của Slint), nên:

- Không có hiệu ứng làm mờ. Nền mờ của màn xem trước được tạo trong Rust (`wallpaper_host::ambient`): thu ảnh về vài điểm ảnh rồi phóng lên lại.
- Ô ảnh trong lưới là ảnh đã cắt và bo góc sẵn, không trong suốt, để cuộn chỉ là chép điểm ảnh. Lớp phủ khi rê chuột chỉ vẽ cho một ô.
- Không xoay hay co giãn phần tử. Chuyển động chỉ gồm đổi màu và trượt thẻ.

Thanh tiêu đề của Windows được đổi sang màu nền của app (`tint_title_bar`). Windows 10 chỉ nhận thanh tiêu đề tối, không nhận màu.

## Số đo sau khi làm lại

Trên máy dev, cùng cách đo như trước:

| | Trước | Sau |
|---|---|---|
| Vẽ một khung hình khi cuộn, 2560×1369 | 3,3 ms | 3,9 ms |
| Vẽ một khung hình khi cuộn, 3840×2100 | 8–9 ms | 7,7 ms |
| RAM private khi cửa sổ mở | 13–14 MB | 14–15 MB |
| File exe | 12,4 MB | 14,9 MB |

## Xem nhanh mọi trạng thái

```sh
cargo run --release --example readme -- ../docs/images <folder khác> 1120 600 1
```

Folder thứ hai nhận thêm ảnh của các trạng thái không đưa vào README: rê chuột, thông báo, yêu thích, chưa chọn thư mục.

Giao diện mới, gồm các nút khi rê chuột trong cửa sổ thật, đã được thử tay trên máy dev (26/11/2026).

## Chưa làm

- Giao diện sáng. App chỉ có giao diện tối.
- Điều hướng bằng bàn phím trong lưới.
- Chưa xem trên Windows 10: font `Segoe UI Variable` không có sẵn ở đó, app sẽ dùng font mặc định của hệ thống.
