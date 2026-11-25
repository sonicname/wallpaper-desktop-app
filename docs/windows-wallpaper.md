# Đặt hình nền trên Windows

Những gì đã kiểm chứng trên máy thật ngày 25/11/2026 (Windows 11 26H2 build 26300, một màn hình 2560×1440). Chưa thử Windows 10, nhiều màn hình thật, Spotlight, slideshow hay máy có policy.

## Cách app làm

1. Tải ảnh gốc, cắt và thu về đúng số pixel vật lý của từng màn hình.
2. Lưu PNG, tên theo nội dung. PNG quá 20 MB thì chuyển sang JPEG chất lượng 97.
3. Đọc trạng thái hiện tại và ghi vào lịch sử **trước khi** đổi. Ảnh của chương trình khác được sao một bản, để hoàn tác được cả khi file gốc bị xóa.
4. Đặt qua `IDesktopWallpaper` rồi chờ tới khi đọc lại đúng. Nếu lỗi, trả lại trạng thái cũ.

Lịch sử giữ tối đa 10 bước **cho mỗi virtual desktop**, và không bỏ bước đầu tiên (hình nền trước khi dùng app). Riêng khi có hơn 20 desktop thì lịch sử của desktop lâu không đổi nhất bị xóa.

## Virtual desktop

Windows chỉ cho API công khai đổi hình nền của desktop đang đứng, nên app gắn mọi thay đổi với desktop lúc người dùng bấm nút:

- Mã desktop lấy từ `IVirtualDesktopManager`: hỏi cửa sổ đang đứng trước, rồi tới các cửa sổ khác đang hiện. Desktop trống hoàn toàn thì không có câu trả lời.
- Hoàn tác chỉ dùng lịch sử của desktop hiện tại.
- Chuyển desktop trong lúc tải ảnh: không đổi gì và báo cho người dùng.
- Chuyển desktop đúng lúc đang đổi: không rollback, vì rollback sẽ rơi vào desktop mới. Ảnh và bước hoàn tác được giữ lại.
- Ảnh đang hiện ở desktop khác không bị dọn.

## Đã kiểm chứng

| Điều | Kết quả |
|---|---|
| PNG có bị nén lại không | Không. Bản Windows tạo giống từng pixel bản của app (226.188 giá trị màu lấy mẫu, sai lệch 0) |
| `SetWallpaper` trả về khi nào | Sau khoảng 2 ms, nhưng đọc lại vẫn ra ảnh cũ trong khoảng **535 ms** |
| Đặt từng màn hình so với đặt chung (`NULL`) | Khác nhau bên trong: đặt từng màn hình làm registry `WallPaper` trỏ tới `TranscodedWallpaper` và thêm `TranscodedImageCache_000`. Vì vậy app dùng `NULL` khi mọi màn hình chung một ảnh |
| Hoàn tác | Đúng như cũ: registry, kiểu đặt ảnh và nội dung `TranscodedWallpaper` |
| Virtual desktop | Chỉ desktop hiện tại đổi, 4 desktop còn lại giữ nguyên |
| Tên màn hình | Tên từ `EnumDisplayDevicesW` (`EDD_GET_DEVICE_INTERFACE_NAME`) trùng với tên của `IDesktopWallpaper` |

## Hạn chế đã biết

- Hoàn tác sau khi chuyển desktop đã được thử tay trên máy thật (25/11/2026). Trường hợp chuyển desktop trong lúc đang tải hoặc đang đổi mới chỉ thử bằng desktop giả lập. Chưa biết ngay sau khi chuyển, Windows có trả về hình nền của desktop cũ trong chốc lát hay không.
- Nếu ghim cửa sổ app lên mọi desktop thì Windows không cho biết mã của nó; app dựa vào các cửa sổ khác.
- Spotlight và slideshow bị tắt khi đặt ảnh và hoàn tác không bật lại được. App cảnh báo trước; giá trị nhận biết Spotlight (`BackgroundType = 3`) chưa được kiểm chứng.
- Màn hình không có mặt lúc hoàn tác sẽ bị bỏ qua.
