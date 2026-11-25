# Đổi hình nền mỗi ngày

Code: `project/packages/daily` (lịch, hàng đợi, chống lặp) và `project/apps/desktop/src/controller/rotation.rs` (nối với giao diện và tray).

## Khi nào đổi

App hỏi mỗi phút một câu: "hôm nay đã đổi chưa?". Hỏi lại bao nhiêu lần cũng không đổi hai lần.

- Một ngày bắt đầu từ giờ đổi người dùng chọn (mặc định 08:00), không phải từ nửa đêm.
- Máy tắt nhiều ngày thì chỉ đổi một lần khi mở lại.
- Đồng hồ bị chỉnh lùi hoặc đổi múi giờ: không đổi lại. Đồng hồ từng chạy trước thì app tự sửa, không chờ tới ngày đó.
- Bật lần đầu thì đổi ngay.
- Hoãn khi Windows báo đang toàn màn hình, chơi game hoặc trình chiếu.
- Hoãn khi không biết đang ở virtual desktop nào (desktop trống, có nhiều desktop), vì khi đó không hoàn tác đúng chỗ được. Chỉ đổi desktop đang đứng.
- Đổi thất bại (ví dụ máy bị khóa hình nền) thì thử lại sau 15 phút, hoặc ngay khi đổi cài đặt.

## Chọn ảnh

- Mỗi kích thước màn hình có hàng đợi riêng 3 ảnh, đã cắt sẵn. Lúc đổi không cần mạng.
- Lấy khoảng 48 ảnh chưa dùng đầu tiên theo kiểu sắp xếp đã chọn, rồi chọn ngẫu nhiên trong đó.
- Bỏ ảnh phải phóng to. Chỉ nhận ảnh bị cắt tối đa 12%; không có thì nới tới 30%.
- Ảnh đã đưa vào hàng đợi không lặp lại (nhớ 3.000 ảnh). Khi nguồn cạn thì nửa cũ nhất được dùng lại. Ảnh bị "không hiện lại" thì không bao giờ.
- Đổi bộ lọc thì bỏ hàng đợi cũ. Những ảnh đó chưa hiện nên vẫn được chọn lại.
- Tải lỗi thì chờ 2, 4, 8, 16 rồi 30 phút.

## Chạy nền

- Chỉ chạy nền khi đã bật tự đổi. Khi đó nút đóng cửa sổ đưa app xuống khay; nếu không bật thì đóng là thoát.
- Mở app lần nữa thì cửa sổ của bản đang chạy hiện lên.
- Khởi động cùng Windows dùng khóa `Run` của người dùng, mặc định tắt. Nếu người dùng đã chặn trong Task Manager thì app chỉ báo, không ghi đè.

Đo trên máy dev (25/11/2026, Windows 11, cửa sổ 1280×820):

| Trạng thái | RAM private |
|---|---|
| Khởi động ẩn, chưa mở cửa sổ | 3,3 MB (CPU 0 ms trong 65 giây) |
| Cửa sổ đang mở | 16 MB |
| Đóng xuống khay sau khi đã mở | 13–14 MB, không tăng qua 6 lần đóng mở |

Mở cửa sổ từ khay mất 31–49 ms, chưa tính thời gian ảnh hiện ra.

Chuẩn bị 3 ảnh từ Wallhaven mất 5,7–7,7 giây cho mỗi dạng 1920×1080, 3840×2160, 3440×1440 và 1440×2560.

## Chưa làm hoặc chưa thử

- Chưa tránh mạng tính phí (metered) và chế độ tiết kiệm pin.
- Chưa có chế độ không thường trú bằng Task Scheduler.
- Đã thử tay trên máy thật (25/11/2026). Chưa thử nhiều màn hình thật, và chưa chạy qua nhiều ngày thật: lịch mới thử bằng đồng hồ giả.
