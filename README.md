# Portly

Sistemdeki aktif TCP portlarını gerçek zamanlı listeleyen, gerektiğinde tek tıkla sonlandırabilen hafif Windows sistem tepsisi uygulaması.

![Platform](https://img.shields.io/badge/platform-Windows-blue)
![Tauri](https://img.shields.io/badge/Tauri-2.x-orange)
![License](https://img.shields.io/badge/license-MIT-green)

---
<img width="380" height="498" alt="image" src="https://github.com/user-attachments/assets/be7c0f11-3386-409b-9351-7b7a4b537d73" />


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

1. GitHub Releases sayfasına git  
2. En son sürümden `portly_x.x.x_x64-setup.exe` dosyasını indir  
3. Çalıştır → UAC onayı ver → kurulum tamamlanır  

> ⚠️ Tüm sistem portlarını görmek için uygulamayı **Run as Administrator** ile çalıştır.

---

### CLI ile Kaynak Koddan Kurulum

#### Gereksinimler
```
node --version  
rustup --version
```

Eksikse kur:

```
winget install OpenJS.NodeJS  
winget install Rustlang.Rustup
```

---

### Build Al ve Kur

```
git clone https://github.com/erdincdegirmenci/portly.git  
cd portly  
npm install  
npm run tauri build  
```
---

### Çıktılar

NSIS Installer:
```
src-tauri/target/release/bundle/nsis/portly_0.1.0_x64-setup.exe
```

MSI:
```
src-tauri/target/release/bundle/msi/portly_0.1.0_x64_en-US.msi
```

Executable:
src-tauri/target/release/portly.exe  

---

### Geliştirici Modu
```
git clone https://github.com/erdincdegirmenci/portly.git  
cd portly  
npm install  
npm run tauri dev  

```
---

## Kullanım

| Eylem | Açıklama |
|------|----------|
| Uygulamayı aç / gizle | Tray ikonuna sol tıkla |
| Port ara | Port no / process adı / dizin yaz |
| Process sonlandır | ✕ butonuna tıkla |
| Terminal aç | >_ butonu |
| Klasör aç | 📁 butonu |
| Yenileme | 3s / 5s / 10s / 30s |

---

## Sistem Gereksinimleri

| Gereksinim | Detay |
|------|--------|
| OS | Windows 10 / 11 (x64) |
| WebView2 | Windows 11’de hazır, Windows 10 için kurulum gerekir |
| Yetki | Administrator önerilir |

WebView2: https://developer.microsoft.com/en-us/microsoft-edge/webview2/

---

## Sorun Giderme

| Sorun | Çözüm |
|------|------|
| Portlar görünmüyor | Admin olarak çalıştır |
| Tray ikon yok | Sistem tepsisini kontrol et |
| Kill sonrası port kalıyor | Refresh yap |
| WebView2 hatası | Runtime kur |
| cargo not found | Rust toolchain kur (winget install Rustlang.Rustup) |

---

## Lisans

MIT
