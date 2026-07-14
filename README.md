# GreenIEM

Cửa hàng bán IEM, dongle DAC/AMP và phụ kiện âm thanh. Frontend viết bằng [Yew](https://yew.rs)
(biên dịch sang WebAssembly bằng [Trunk](https://trunkrs.dev)), backend bằng [Axum](https://github.com/tokio-rs/axum),
dữ liệu lưu trong MongoDB. Cả hai được đóng gói và chạy trong cùng một container/server.

## Cấu trúc project

```
shared/     DTO dùng chung giữa frontend và backend (serde only, không phụ thuộc wasm/tokio)
frontend/   Ứng dụng Yew (build bằng trunk -> thư mục dist/ ở gốc repo)
backend/    API Axum: MongoDB, xác thực admin, rate-limit + captcha, phục vụ luôn dist/
Dockerfile  Multi-stage build: build frontend + backend, đóng gói vào image runtime
fly.toml    Cấu hình deploy lên Fly.io
```

## Chạy local

1. Cài Rust + target wasm32 + Trunk (chỉ cần làm 1 lần):
   ```bash
   rustup target add wasm32-unknown-unknown
   cargo install trunk wasm-bindgen-cli
   ```
2. Copy `.env.example` thành `.env` và điền `MONGODB_URI`, `JWT_SECRET` (xem hướng dẫn trong file).
3. Chạy backend (đọc `.env`, mặc định lắng nghe cổng 3000 theo cấu hình dev):
   ```bash
   cargo run -p backend
   ```
4. Ở terminal khác, chạy frontend (Trunk tự proxy `/api/*` sang backend, xem `frontend/Trunk.toml`):
   ```bash
   cd frontend
   trunk serve
   ```
5. Mở `http://127.0.0.1:8080`. Trang quản trị (ẩn) nằm ở `/console` — lần đầu vào sẽ hiện form
   tạo tài khoản admin vì database chưa có tài khoản nào.

## Bảo mật

- Mật khẩu admin băm bằng Argon2id; phiên đăng nhập dùng JWT lưu trong cookie HttpOnly + Secure + SameSite=Lax.
- Rate-limit + captcha toán học đơn giản: quá 10 request/10 giây từ một IP sẽ bị yêu cầu giải captcha
  trước khi tiếp tục gọi API (`backend/src/security/rate_limit_captcha.rs`).
- Header bảo mật cơ bản (CSP, X-Frame-Options, X-Content-Type-Options...), giới hạn dung lượng body request,
  giá đơn hàng luôn được tính lại từ database (không tin giá client gửi lên).

## Triển khai lên Fly.io

```bash
fly launch --no-deploy   # hoặc fly apps create <tên-app>, rồi sửa `app` trong fly.toml
fly secrets set MONGODB_URI="..." JWT_SECRET="$(openssl rand -hex 32)"
fly deploy
```

`Dockerfile` build cả frontend (trunk) lẫn backend (cargo) trong một image, chạy chung một service ở cổng 8080.
