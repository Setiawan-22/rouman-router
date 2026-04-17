# 📑 ARSITEKTUR rouman ( router mandala ) ROUTER OS
# Visi: Next-Generation Firewall (NGFW) & SD-WAN berbasis Linux + Rust + svelte

## LAYER 1: FONDASI OS (The Base)
- Network Stack: Native Linux Kernel.
- Pembaruan Sistem: A/B Partitioning (seperti OSTree) untuk rollback otomatis jika sistem gagal booting pasca-update (Anti-Brick).
- Mode Jaringan Fisik: Terhubung ke Modem/ONT ISP dalam status "Bridge Mode". rouman bertindak sebagai pemegang kendali jaringan utama (termasuk sebagai PPPoE Client jika diperlukan).

## LAYER 2: DATA PLANE (Pemrosesan Paket)
- Standard Routing & NAT: Dikelola oleh kernel Linux via instruksi Netlink.
- High-Speed / L7 Filtering: eBPF (XDP) untuk Deep Packet Inspection (DPI), pemblokiran aplikasi spesifik, dan mitigasi ancaman dengan latensi secepat hardware.
- Threat Prevention (eBPF DPI): Pemfilteran paket berbasis IP Reputasi dan inspeksi SNI (Server Name Indication) untuk memblokir lalu lintas iklan/malware tingkat lanjut yang mem-bypass DNS (Meniru fitur Sophos Web Filtering/ATP).
- Kriptografi & Tunneling: WireGuard native (in-tree kernel) didukung akselerasi hardware otomatis (AES-NI / ARM Crypto).
- Traffic Shaping (QoS): Modul kernel CAKE (Common Applications Kept Enhanced) atau fq_codel untuk mencegah bufferbloat dan manajemen prioritas antrean bandwidth.

## LAYER 3: CONTROL PLANE (Rust Core Daemon)
*Inti sistem operasi yang ditulis secara khusus menggunakan ekosistem Rust.*
- Configuration Engine: Sistem manajemen konfigurasi transaksional (Candidate Config -> Commit -> Confirm -> Auto-Rollback).
- DHCP & Network Orchestrator: Mengelola konfigurasi DHCP server secara dinamis dan membaca status pembagian IP secara real-time.
- State Machine & HA (High Availability): Sinkronisasi state firewall (conntrack) antar-router fisik via gRPC latensi sub-milidetik.
- MAC-Based Discovery: Listener Layer-2 (via libpnet) untuk akses manajemen darurat saat IP tidak terkonfigurasi (Pengganti Mikrotik RoMON / MAC Telnet).

## LAYER 4: MANAGED SERVICES (Layanan Tersertifikasi)
*Layanan/aplikasi esensial yang siklus hidupnya dikendalikan penuh oleh Rust Daemon.*
- DNS Sinkhole Engine: Modul Rust yang secara otomatis mengambil, mem-parsing, dan memperbarui ribuan daftar blokir domain (blocklist komunitas) ke dalam DNS Server lokal untuk memblokir iklan dan pelacak di seluruh jaringan secara transparan.
- Zero Trust & Remote Access: Layanan cloudflared terintegrasi untuk tunneling aman tanpa Port Forwarding (Bypass IP Dinamis/CGNAT ISP).
- Secure DNS & DHCP Server: dnsmasq / Kea DHCP (untuk DHCP Server terpusat) dipadukan dengan Hickory DNS (untuk memaksa DoH/DoT ke Cloudflare/Quad9).
- Dynamic Routing: FRRouting (FRR) di latar belakang untuk menangani protokol BGP / OSPF.
- Event Scripter: Embedded Lua (mlua) atau Rhai engine untuk eksekusi skrip otomasi berdasarkan kondisi jaringan (Pengganti Mikrotik Netwatch / Scheduler).
- Telemetry & Logging: Parser log lokal menjadi endpoint monitoring tools nanti

## LAYER 5: ANTARMUKA MANAJEMEN (Management Interfaces)
- API Gateway: Backend axum (REST/gRPC) untuk menerima instruksi jarak jauh dari platform orkestrasi pusat.
- Command Line Interface (CLI): Antarmuka terminal interaktif (dibangun dengan rustyline) untuk eksekusi perintah secara langsung dengan fitur auto-completion.
- Web / Desktop GUI: (Ekspansi masa depan) Antarmuka visual yang langsung berkomunikasi dengan API Gateway lokal rouman.