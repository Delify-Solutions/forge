<div align="center">

# Delify Forge

**Môi trường phát triển web local thế hệ mới, native và mã nguồn mở.**

[![License: AGPL v3](https://img.shields.io/badge/License-AGPL_v3-blue.svg)](https://www.gnu.org/licenses/agpl-3.0)
[![Platform: macOS](https://img.shields.io/badge/Platform-macOS_14%2B-lightgrey.svg)](#)
[![Status: Alpha](https://img.shields.io/badge/Status-Alpha-orange.svg)](#)

[English](README.md) · [Tiếng Việt](README.vi.md)

</div>

---

## Delify Forge là gì

Delify Forge là một ứng dụng desktop mã nguồn mở, biến máy local của bạn thành môi trường phát triển web nhanh và đáng tin cậy. App quản lý các engine (web server, runtime ngôn ngữ, database) và toàn bộ phần kết nối giữa chúng (DNS, socket, config) để bạn tập trung vào việc làm sản phẩm.

Forge dành cho developer muốn **độ hoàn thiện của Laravel Herd, độ rộng của XAMPP, và sự cởi mở của một công cụ thuộc về cộng đồng** — gói gọn trong một app native duy nhất.

## Tại sao cần thêm một local dev tool nữa

| Tool | Open source | Đa ngôn ngữ | Nhiều web server | Native | Giá |
|------|:-:|:-:|:-:|:-:|:-:|
| **Delify Forge** | ✅ AGPLv3 | ✅ | ✅ | ✅ Tauri | Miễn phí |
| Laravel Herd | ❌ | ❌ chỉ PHP | Hạn chế | ✅ | Free + Pro |
| XAMPP | Mixed | ❌ chỉ PHP | ❌ | Bundled | Miễn phí |
| Laravel Valet | ✅ MIT | ❌ chỉ PHP | ❌ chỉ Nginx | ✅ CLI | Miễn phí |
| MAMP | ❌ | ❌ | ❌ | ✅ | Free + Pro |

Forge lấp khoảng trống: một local dev environment open-source, native, đa ngôn ngữ, đa web server.

## Trạng thái

**Pre-MVP.** Repo đang được scaffold. Bản release đầu tiên (`v0.0.1-mvp`) target macOS 14+, hỗ trợ Nginx và 1 phiên bản PHP. Xem [roadmap](#roadmap) phía dưới.

## Tính năng dự kiến

### MVP
- Hỗ trợ macOS 14+
- Thêm dự án từ folder
- Domain `*.test` được route tự động qua dnsmasq + `/etc/resolver/test`
- Vòng đời Nginx + PHP-FPM được app quản lý
- UI sidebar lấy cảm hứng từ Laravel Herd, dark theme mặc định

### Roadmap

| Phase | Highlights |
|-------|------------|
| **MVP** | Nginx, 1 phiên bản PHP, domain `.test`, add/list/remove site |
| **V0.2** | Multi PHP version qua mise, alias domain, project scaffolding (PHP/Laravel) |
| **V0.3** | Apache, OpenLiteSpeed, PHP extensions manager, download binary bundled |
| **V0.4** | Node.js, framework templates (Next, Vite), Nginx-as-gateway proxy mode |
| **V0.5** | Database manager (DBngin-style spawn cho MySQL, MariaDB, PostgreSQL, Redis...) |
| **V0.6** | Database GUI (clean-room TablePro-style implementation) |
| **V0.7** | API tester (Bruno-based), tab cron |
| **V1.0** | Polish, hỗ trợ Linux |
| **V2.0** | Hỗ trợ Windows, AI features, plugin system |

## Tech stack

- **Tauri 2** với Rust core và WebView native theo OS
- **React 18 + TypeScript + Tailwind CSS 4** với **shadcn/ui**
- **SQLite** làm source of truth, **Tera** sinh config
- **mise** quản lý phiên bản ngôn ngữ
- **Bruno** (dự kiến) cho API tester

## Cài đặt

> Chưa có bản build chính thức. Mục này sẽ được cập nhật khi `v0.0.1-mvp` được tag.

Khi MVP ra mắt, bạn sẽ cài được bằng:

```bash
# qua Homebrew Cask (dự kiến)
brew install --cask delify-forge

# hoặc tải DMG từ trang Releases
```

Hiện tại, chạy từ source:

```bash
git clone https://github.com/Delify-Solutions/forge
cd forge
pnpm install
pnpm tauri dev
```

Yêu cầu môi trường: Rust (1.78+), Node.js (20+), pnpm (8+), Homebrew với `nginx` và `php` có sẵn trong `PATH` cho MVP.

## Tổng quan kiến trúc

```
┌──────────────────────────────────────────────────────┐
│ Tauri WebView (React + TypeScript + Tailwind)        │
│  Sidebar │ Sites │ PHP │ Services │ About            │
└────────────┬─────────────────────────────────────────┘
             │ Tauri IPC
┌────────────▼─────────────────────────────────────────┐
│ Rust core (tokio async runtime)                      │
│                                                      │
│  ┌────────────┐  ┌──────────┐  ┌─────────────────┐  │
│  │  SQLite    │  │  Tera    │  │  Platform trait │  │
│  │  store     │──▶ templates│──▶ DnsManager      │  │
│  │  (truth)   │  │          │  │ ProcessSupervisor│ │
│  └────────────┘  └──────────┘  │ PathProvider    │  │
│                                 └────────┬────────┘  │
│                                          │           │
│                            ┌─────────────┼───────────┴─┐
│                            │  macos.rs (impl)         │
│                            │  windows.rs (stub)       │
│                            │  linux.rs (stub)         │
│                            └─────────────┬─────────────┘
└──────────────────────────────────────────┼─────────────
                                            │
                              spawn / supervise / signal
                                            ▼
                       ┌────────────────────────────────┐
                       │ Nginx │ Apache │ OLS │ PHP-FPM │
                       │ dnsmasq @ :5353               │
                       └────────────────────────────────┘
```

Cấu trúc cross-platform được đặt sẵn từ ngày đầu, dù MVP chỉ ship implementation cho macOS.

## Mô hình quyền

Delify Forge cần quyền admin **đúng một lần** trong first-run setup, để ghi `/etc/resolver/test`. MVP dùng `osascript` để hiện native password dialog của macOS — cùng pattern với Laravel Herd, chỉ khác là chưa có signed helper qua Apple Developer cert. Sudo không bị giữ lại, và bản thân app chạy hoàn toàn ở user space.

Privileged helper signed kiểu `LaunchDaemon` đang nằm trong roadmap, sẽ làm khi dự án có Apple Developer certificate.

## Đóng góp

Issues, ý tưởng, và pull requests đều được chào đón. Vui lòng đọc [CONTRIBUTING.md](CONTRIBUTING.md) trước khi submit thay đổi — Delify Forge dùng license **AGPLv3**, có ràng buộc với derivative works.

## License

Copyright (C) 2026 Delify Solutions.

Delify Forge là phần mềm tự do: bạn có thể phân phối lại và/hoặc sửa đổi nó dưới các điều khoản của **GNU Affero General Public License**, phiên bản 3 hoặc (tùy chọn) bất kỳ phiên bản nào sau đó, được công bố bởi Free Software Foundation. Xem [LICENSE](LICENSE) để biết toàn văn.

Chương trình này được phân phối với hy vọng sẽ hữu ích, nhưng **KHÔNG CÓ BẤT KỲ BẢO ĐẢM NÀO**; ngay cả bảo đảm ngầm về KHẢ NĂNG THƯƠNG MẠI hay PHÙ HỢP CHO MỘT MỤC ĐÍCH CỤ THỂ.

## Lời cảm ơn

Delify Forge đứng trên vai những dự án đi trước. Chúng tôi học hỏi — nhưng không copy code — từ các project sau:

- **Laravel Herd** cho UX và pattern privileged-helper
- **Laravel Valet** cho catch-all nginx + driver model
- **XAMPP** cho khái niệm bundled-engine
- **TablePro** cho cảm hứng source-of-truth và plugin architecture
- **DBngin** cho việc spawn database engine on-demand
- **aaPanel** cho pattern multi-webserver gateway

— **Delify Solutions**
