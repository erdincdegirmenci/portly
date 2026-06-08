# Portly

Sistemdeki aktif TCP portlarını gerçek zamanlı listeleyen, gerektiğinde tek tıkla sonlandırabilen hafif Windows sistem tepsisi uygulaması.

![Platform](https://img.shields.io/badge/platform-Windows-blue)
![Tauri](https://img.shields.io/badge/Tauri-2.x-orange)
![License](https://img.shields.io/badge/license-MIT-green)

---

## Özellikler

- Aktif TCP portlarını process adı ve dizin bilgisiyle listeler
- Port / process / dizin adına göre anlık arama
- Tek tıkla process sonlandırma (onay ekranıyla)
- Seçilen dizinde terminal veya Explorer açma
- Otomatik yenileme (3s / 5s / 10s / 30s)
- Sistem tepsisinde çalışır, görev çubuğunu meşgul etmez

---

## Kurulum

### Hazır `.exe` ile (Önerilen)

1. [Releases](https://github.com/erdincdegirmenci/portly/releases) sayfasına git
2. En son sürümden `portly_x.x.x_x64-setup.exe` dosyasını indir
3. Çalıştır → UAC onayı ver → kurulum tamamlanır

> ⚠️ Tüm sistem portlarını görmek için uygulamayı **Run as Administrator** ile çalıştır.

---

### CLI ile Kaynak Koddan Kurulum

#### Gereksinimler

node --version rustup --version


Eksikse kur:

winget install OpenJS.NodeJS winget install Rustlang.Rustup

#### Build Al ve Kur
git clone https://github.com/erdincdegirmenci/portly.git cd portly npm install npm run tauri build

Build tamamlandıktan sonra:

Installer
.\src-tauri\target\release\bundle\nsis\portly_0.1.0_x64-setup.exe
MSI
.\src-tauri\target\release\bundle\msi\portly_0.1.0_x64_en-US.msi

.\src-tauri\target\release\portly.exe

> İlk build ~10 dakika sürebilir.

---

### Geliştirici Ortamı

git clone https://github.com/erdincdegirmenci/portly.git cd portly npm install npm run tauri dev

---

## Kullanım

| Eylem | Nasıl |
|---|---|
| Uygulamayı aç / gizle | Tray ikonuna sol tıkla |
| Port ara | Arama kutusuna port no, process adı veya dizin yaz |
| Process sonlandır | **[✕]** butonuna tıkla → onayla |
| Terminalde aç | **[>_]** butonuna tıkla |
| Klasörde aç | **[📁]** butonuna tıkla |
| Yenileme aralığı | Options → 3s / 5s / 10s / 30s |

---

## Sistem Gereksinimleri

| | |
|---|---|
| **İşletim Sistemi** | Windows 10 / 11 (x64) |
| **WebView2 Runtime** | Windows 11'de önceden yüklü — Windows 10 için [buradan indir](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) |
| **Tam erişim** | Administrator yetkisi önerilir |

---

## Sorun Giderme

| Sorun | Çözüm |
|---|---|
| Bazı portlar görünmüyor | Sağ tık → **Run as Administrator** |
| Uygulama görev çubuğunda yok | Sistem tepsisine bak (saat yanı) |
| Kill sonrası port hâlâ görünüyor | **[↺]** refresh butonuna bas |
| WebView2 hatası | [WebView2 Runtime](https://developer.microsoft.com/en-us/microsoft-edge/webview2/)'ı kur |
| `cargo not found` | `winget install Rustlang.Rustup` → terminali yeniden aç |

---

## Lisans

MIT