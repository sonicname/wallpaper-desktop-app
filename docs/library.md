# Yêu thích và thư mục trên máy

Code: `project/packages/favorites`, `project/packages/provider-folder` và `project/apps/desktop/src/controller/library.rs`.

App có ba loại nguồn, hiện thành các tab ở góc trái: nguồn trên mạng (Wallhaven), **Yêu thích** và **Thư mục**. Cả ba đều là `Provider`, nên duyệt, xem trước, đặt hình nền và đổi hằng ngày dùng chung một đường code. Mỗi nguồn nhớ bộ lọc riêng trong lúc app chạy.

## Yêu thích

- Bấm hình trái tim trong màn xem trước, hoặc phím `F`. Ảnh yêu thích có trái tim trong lưới.
- Lưu ở `%LOCALAPPDATA%\WallpaperDesktop\favorites`: danh sách trong `favorites.json`, kèm một bản thumbnail cho mỗi ảnh. Nhờ đó tab Yêu thích hiện được khi mất mạng và sau khi cache bị dọn.
- **Ảnh gốc không được sao lại.** Khi xem trước hoặc đặt làm hình nền, app tải lại từ nguồn (hoặc đọc file nếu là ảnh trên máy). Nguồn gỡ ảnh, hoặc file bị xóa, thì ảnh yêu thích đó không dùng được nữa.
- Ảnh trên máy đã bị xóa thì ẩn khỏi danh sách; file quay lại thì ảnh hiện lại.
- Mặc định không lọc theo màn hình, vì đây là ảnh người dùng đã chọn.

## Thư mục

- Chọn bằng hộp thoại thư mục của Windows. App nhớ một thư mục, lưu trong `folder.txt`.
- Nhận JPEG, PNG, WebP và BMP; tìm cả thư mục con, sâu tối đa 4 cấp, tối đa 20.000 file.
- Tìm kiếm theo tên file. Sắp xếp mới nhất (theo ngày sửa) hoặc ngẫu nhiên.
- Kích thước ảnh đọc từ phần đầu file, 60 ảnh mỗi trang. File hỏng bị bỏ qua.
- Ảnh có cạnh dài hơn 640 px được tạo một bản nhỏ trong cache `thumbs`, tạo lại khi file thay đổi. App không sửa, không xóa file nào trong thư mục.
- Thư mục bị xóa hoặc ổ đĩa bị rút thì app báo rõ thay vì hiện danh sách trống.

## Đổi hằng ngày

Bảng "Mỗi ngày" có mục **Nguồn ảnh**. Với Yêu thích và Thư mục, chỉ ảnh đủ lớn và đúng tỉ lệ màn hình mới được dùng (không phóng to, cắt tối đa 30%). Ít ảnh thì xoay vòng. Bỏ một ảnh khỏi yêu thích thì ảnh đó cũng bị bỏ khỏi hàng đợi.

## Số đo

Trên máy dev, với thư mục hình nền có sẵn của Windows (`C:\Windows\Web`, 36 ảnh JPEG tới 3840×2400):

| Việc | Thời gian |
|---|---|
| Liệt kê cả thư mục, lần đầu | 0,23 giây |
| Tạo bản nhỏ từ một ảnh 3840×2400 | khoảng 35 ms |
| Vẽ một ô từ bản nhỏ | khoảng 1 ms |

## Chưa làm hoặc chưa thử

- Chưa có "thích" ảnh đang làm hình nền từ khay hệ thống.
- Chỉ một thư mục; chưa kéo thả file vào app.
- Chưa thử với thư mục hàng nghìn ảnh, ổ mạng, hay ảnh rất lớn (trên 100 MB bị từ chối).
- Chọn thư mục thật, thích ảnh và đổi hằng ngày từ thư mục đã được thử tay trên máy dev (26/11/2026).
