# xv8-rust-posix 版本規劃 (v2.x) — 網路功能

> v2.x = 所有網路相關功能。需先完成 v1.1（POSIX 工具整合）具備基礎環境。

---

## v2.1 — BSD Socket API

> TCP/UDP Socket 核心：讓 xv8 成為網路節點。

### 功能項目

- [ ] `sys_socket(domain, type, protocol)` — 建立 socket（AF_INET, SOCK_STREAM/ DGRAM）
- [ ] `sys_bind(fd, addr, addrlen)` — 綁定 IP:Port
- [ ] `sys_connect(fd, addr, addrlen)` — 連線（TCP client）
- [ ] `sys_listen(fd, backlog)` — 監聽（TCP server）
- [ ] `sys_accept(fd, addr, addrlen)` — 接受連線
- [ ] `sys_sendto(fd, buf, len, flags, addr, addrlen)` — 發送
- [ ] `sys_recvfrom(fd, buf, len, flags, addr, addrlen)` — 接收
- [ ] `sys_sendmsg` / `sys_recvmsg` — 結構化訊息（可選）
- [ ] `sys_shutdown(fd, how)` — 關閉連線
- [ ] `sys_setsockopt(fd, level, optname, optval)` — TCP_NODELAY, SO_KEEPALIVE...
- [ ] `sys_getsockopt(fd, level, optname, optval)` — 讀取選項
- [ ] `sys_getpeername` / `sys_getsockname` — 取得遠端/本地端位址

### 測試
- [ ] `_posix_udp` — UDP 收發測試
- [ ] TCP echo server + client 互通
- [ ] `netstat` 顯示 socket 狀態

### 交付
TCP/UDP client 和 server 正常運作。

---

## v2.2 — 網路工具第一波（TCP 工具）

### 工具

| 工具 | 說明 |
|------|------|
| nc / netcat | 網路瑞士刀 |
| telnet | 遠端登入（純文字） |
| curl | HTTP client（可選） |
| wget | HTTP 下載（可選） |

### 功能項目

- [ ] `sys_select` — I/O 多工（select/fd_set）
- [ ] `sys_poll` — POSIX poll（可選）
- [ ] `sys_epoll_create` / `sys_epoll_ctl` / `sys_epoll_wait` — Linux epoll（可選）

### 交付
`nc -l -p 8080` 聆聽，`telnet localhost 8080` 連線成功。

---

## v2.3 — DHCP + 網路設定

### 功能項目

- [ ] DHCP client — 自動取得 IP（基於 UDP）
- [ ] `setup_net.sh` 改進 — 支援 DHCP 模式
- [ ] `ifconfig` — 網路介面設定
- [ ] `route` — 路由表管理
- [ ] `/etc/networks` — 網路設定檔
- [ ] `/etc/hosts` — 主機名稱解析
- [ ] `hostname` 設定（寫入 /etc/hostname）

### 工具
- [ ] `ifconfig` — 顯示/設定 IP
- [ ] `route` — 顯示/管理路由表
- [ ] `arp` — ARP cache

### 交付
開機後自動透過 DHCP 取得 IP，可 ping 外部主機。

---

## v2.4 — DNS Resolver

### 功能項目

- [ ] `/etc/resolv.conf` — DNS 伺服器設定
- [ ] `gethostbyname` / `gethostbyaddr` — 名稱解析
- [ ] DNS client — 向 DNS 伺服器發送查詢
- [ ] `nslookup` — DNS 查詢工具
- [ ] `host` — DNS 查詢工具
- [ ] `dig` — 詳細 DNS 查詢

### 交付
`nslookup google.com` 返回 IP 位址。

---

## v2.5 — 網路服務第一波

### 工具/服務

| 服務 | 說明 |
|------|------|
| httpd / boa | 簡單 HTTP server |
| ftpd | FTP server |
| tftpd | TFTP server |
| time (daytime) | 時間伺服器（RFC 868） |
| echo | Echo 伺服器（RFC 862） |

### 功能項目

- [ ] `inetd` / `xinetd` — super-daemon（可選）
- [ ] `/etc/inetd.conf` — 服務設定
- [ ] `syslogd` — 網路 syslog（可選）

### 交付
`httpd`  serve靜態網頁，`wget http://xv8/` 下載成功。

---

## v2.6 — 進階網路功能

### 功能項目

- [ ] Raw Socket — 原始封包（ICMP echo, 可選）
- [ ] `sys_packet` — Linux packet socket（可選）
- [ ] `SO_REUSEADDR` — 埠重用
- [ ] `SO_NONBLOCK` / `O_NONBLOCK` — 非阻塞 I/O
- [ ] `sys_sendfile` — 核心層檔案傳輸
- [ ] IPv6 基本支援（可選，視需求）

### 工具
- [ ] `ping` — ICMP echo
- [ ] `traceroute` — 路由追蹤（可選）
- [ ] `tcpdump` / `tshark` — 封包截取（可選）

### 交付
`ping 8.8.8.8` 成功，`traceroute` 可顯示路由。

---

## v2.7 — NFS Client（可選）

### 功能項目

- [ ] NFS v2/v3 client（可選）
- [ ] `mount -t nfs` — 掛載遠端檔案系統
- [ ] `showmount` — 顯示 NFS 分享

### 限制
NFS 複雜，視時間和穩定性決定是否實作。

---

## v2.8 — SSL/TLS（可選）

### 功能項目

- [ ] OpenSSL / rustls 整合（可選）
- [ ] `https://` 支援（curl, wget）
- [ ] TLS server（stunnel, stunnel替代方案）

### 備註
可能需要 v1.5 的動態連結支援。

---

## v2.9 — 網路壓力測試 + 驗證

### 功能項目

- [ ] 長時間網路穩定性測試
- [ ] TCP 連線池測試
- [ ] UDP 可靠性測試
- [ ] 網路效能測試（吞吐量、延遲）

### 交付
網路功能穩定，可持續運作 24 小時無故障。

---

## 版本時間軸

```
v1.9 ─→ v2.1 ─→ v2.2 ─→ v2.3 ─→ v2.4 ─→ v2.5 ─→ v2.6 ─→ v2.7 ─→ v2.8 ─→ v2.9
       BSD      TCP      DHCP+    DNS      網路      進階      NFS     SSL/    網路
       Socket   工具     ifconfig resolver 服務     網路      Client   TLS    壓測
```

---

## 備註

- v2.1 的 BSD Socket API 是所有網路功能的基礎
- `setup_net.sh` 已有 TAP 設備設定，需整合進 xv8
- 網路功能測試需要兩個環境（QEMU + host 或兩台 QEMU）
- v2.x 依賴 v1.1 的基礎環境（syscall 完整）