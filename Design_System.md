# Design System — SpeedFlow

> **Document Status:** Draft  
> **Version:** 0.1  
> **Last Updated:** 3 November 2026  
> **Product:** SpeedFlow  
> **Platform:** Desktop Application  
> **UI Framework:** Slint  
> **Design Direction:** Modern, Minimalist, Functional

---

# 1. Introduction

## 1.1 Purpose

Dokumen ini mendefinisikan sistem desain visual dan interaksi untuk aplikasi SpeedFlow.

Design System digunakan sebagai pedoman dalam membangun seluruh antarmuka SpeedFlow agar memiliki konsistensi visual, pola interaksi, hierarki informasi, dan pengalaman pengguna yang seragam.

Dokumen ini menjadi referensi bagi proses desain UI/UX dan implementasi komponen antarmuka menggunakan Slint.

## 1.2 Product Overview

SpeedFlow merupakan aplikasi desktop download manager yang membantu pengguna mengunduh, mengelola, dan memantau berbagai file melalui satu aplikasi.

Fitur utama produk meliputi:

- Parallel Download.
- Pause, Resume, Cancel, dan Retry.
- Download Queue.
- Real-time Download Monitoring.
- Bandwidth Management.
- Download History.
- File Management.
- Application Settings.

Antarmuka harus mampu menampilkan informasi teknis yang cukup tanpa membuat pengguna merasa sedang menggunakan aplikasi yang kompleks.

## 1.3 Design Goals

1. **Simple:** Memudahkan pengguna memahami dan menggunakan aplikasi.
2. **Informative:** Menampilkan informasi download secara jelas.
3. **Efficient:** Meminimalkan langkah untuk menjalankan tindakan utama.
4. **Consistent:** Menjaga konsistensi antarhalaman dan komponen.
5. **Responsive:** Menyesuaikan tata letak dengan ukuran jendela desktop.
6. **Accessible:** Memastikan informasi dan kontrol dapat digunakan dengan nyaman.

## 1.4 Design Scope

Design System mencakup:

- Color System.
- Typography.
- Spacing and Layout.
- Border and Radius.
- Iconography.
- Navigation.
- Buttons.
- Inputs.
- Download Cards.
- Progress Indicators.
- Data Tables.
- Dialogs and Popovers.
- Notifications.
- Empty States.
- Loading States.
- Error States.
- Light and Dark Themes.

---

# 2. Design Philosophy

## 2.1 Design Direction

SpeedFlow menggunakan pendekatan visual **Modern Minimal Desktop UI**.

Karakteristik visual:

- Tampilan bersih dan tidak berlebihan.
- Menggunakan warna netral sebagai fondasi.
- Menggunakan satu warna aksen utama untuk tindakan penting.
- Memiliki hierarki informasi yang jelas.
- Mengutamakan keterbacaan angka dan status download.
- Menggunakan border dan background surface untuk membedakan area.
- Menghindari dekorasi yang tidak memiliki fungsi.

## 2.2 Design Principles

### 1. Clarity Over Decoration

Setiap elemen visual harus memiliki tujuan yang jelas.

Informasi seperti nama file, progress, ukuran, kecepatan, dan status harus lebih menonjol dibandingkan dekorasi.

### 2. Information Hierarchy

Informasi penting harus mudah ditemukan.

Urutan prioritas informasi download:

1. File name.
2. Download status.
3. Progress.
4. Download speed.
5. File size.
6. Additional information.

### 3. Minimal Interaction

Tindakan utama harus dapat dilakukan dengan langkah seminimal mungkin.

Contohnya, pengguna harus dapat melakukan Pause atau Resume langsung dari item download tanpa membuka halaman detail.

### 4. Progressive Disclosure

Pengaturan teknis lanjutan tidak perlu selalu ditampilkan.

Pengaturan seperti jumlah koneksi paralel, bandwidth limit, dan konfigurasi lainnya dapat ditempatkan pada Settings atau panel tambahan.

### 5. Consistent Feedback

Setiap interaksi harus memberikan respons yang jelas.

Contohnya:

- Tombol memberikan feedback ketika ditekan.
- Progress berubah ketika download berjalan.
- Status berubah ketika download selesai.
- Error memberikan informasi dan tindakan pemulihan.

### 6. Functional Elegance

SpeedFlow harus terlihat modern dan profesional tanpa mengorbankan kemudahan penggunaan.

---

# 3. Visual Foundation

## 3.1 Color System

SpeedFlow menggunakan sistem warna **Neutral + Green Accent (`#78C375`)**.

Warna netral menjadi fondasi utama, sedangkan aksen hijau (`#78C375`) digunakan sebagai identitas visual dan penanda tindakan utama.

### 3.1.1 Brand Colors

| Token | Hex | Purpose |
|---|---|---|
| Brand Primary | `#78C375` | Primary actions |
| Brand Hover | `#65B462` | Hover state |
| Brand Active | `#529F4F` | Pressed state |
| Brand Subtle | `#EFF8EF` | Soft accent background |
| Brand Dark | `#92D68F` | Dark theme accent |

### 3.1.2 Light Theme Colors

| Token | Hex | Purpose |
|---|---|---|
| Background | `#F7F8FA` | Main application background |
| Surface | `#FFFFFF` | Cards and panels |
| Surface Secondary | `#F1F3F6` | Secondary containers |
| Surface Hover | `#F5F7FA` | Hovered surfaces |
| Border | `#E5E8ED` | Default borders |
| Border Strong | `#D5DAE2` | Emphasized borders |
| Text Primary | `#20242C` | Main text |
| Text Secondary | `#697386` | Secondary text |
| Text Tertiary | `#98A1B0` | Muted text |
| Text Disabled | `#B8BEC8` | Disabled text |

### 3.1.3 Dark Theme Colors

| Token | Hex | Purpose |
|---|---|---|
| Background | `#14171C` | Main application background |
| Surface | `#1C2027` | Cards and panels |
| Surface Secondary | `#242A33` | Secondary containers |
| Surface Hover | `#292F39` | Hovered surfaces |
| Border | `#303743` | Default borders |
| Border Strong | `#414B59` | Emphasized borders |
| Text Primary | `#F1F3F6` | Main text |
| Text Secondary | `#A3ACBA` | Secondary text |
| Text Tertiary | `#788292` | Muted text |
| Text Disabled | `#626B79` | Disabled text |

### 3.1.4 Semantic Colors

Warna semantic digunakan untuk menyampaikan status dan feedback.

| Semantic | Light | Dark | Usage |
|---|---|---|---|
| Success | `#16A36A` | `#35C78A` | Completed |
| Warning | `#D99426` | `#F0B44E` | Warning |
| Danger | `#DC4C4C` | `#F07171` | Failed and destructive actions |
| Info | `#3B82F6` | `#60A5FA` | Information |
| Neutral | `#7C8798` | `#A3ACBA` | Queued and inactive |

### 3.1.5 Download Status Colors

| Status | Color | Meaning |
|---|---|---|
| Downloading | Brand Primary | Download sedang berjalan |
| Queued | Neutral | Menunggu antrean |
| Paused | Warning | Download dijeda |
| Completed | Success | Download selesai |
| Failed | Danger | Download gagal |
| Retrying | Warning | Sistem mencoba kembali |
| Cancelled | Neutral | Download dibatalkan |

Status tidak boleh hanya dibedakan berdasarkan warna. Setiap status harus memiliki teks dan, jika relevan, ikon.

---

## 3.2 Typography

### 3.2.1 Primary Typeface

**Font Family:** Inter

Inter digunakan sebagai font utama karena memiliki keterbacaan yang baik pada antarmuka desktop dan informasi berukuran kecil.

Jika Inter tidak tersedia atau tidak dapat dibundel, gunakan fallback sans-serif yang sesuai.

### 3.2.2 Typography Scale

| Token | Size | Weight | Usage |
|---|---:|---|---|
| Display | 30px | Bold | Large dashboard metric |
| Heading 1 | 24px | Semibold | Page title |
| Heading 2 | 20px | Semibold | Section title |
| Heading 3 | 16px | Semibold | Card title |
| Body Large | 15px | Regular | Important content |
| Body | 14px | Regular | Default text |
| Body Small | 13px | Regular | Secondary information |
| Caption | 12px | Regular | Metadata |
| Label | 12px | Medium | Form labels |
| Button | 13px | Semibold | Button text |

### 3.2.3 Typography Rules

- Gunakan maksimal tiga tingkatan ukuran teks dalam satu komponen.
- Hindari penggunaan font weight terlalu banyak.
- Gunakan angka yang mudah dibaca untuk speed dan file size.
- Gunakan tabular numerals untuk angka yang berubah secara real-time.
- Nama file panjang harus menggunakan truncation dengan akses untuk melihat nama lengkap.

---

## 3.3 Spacing System

SpeedFlow menggunakan basis spacing 4px.

| Token | Value | Usage |
|---|---:|---|
| `space-1` | 4px | Tight spacing |
| `space-2` | 8px | Small spacing |
| `space-3` | 12px | Component spacing |
| `space-4` | 16px | Default spacing |
| `space-5` | 20px | Medium spacing |
| `space-6` | 24px | Section spacing |
| `space-8` | 32px | Large spacing |
| `space-10` | 40px | Page spacing |
| `space-12` | 48px | Major separation |

### Spacing Guidelines

- Padding utama halaman: 24px.
- Padding card: 16px.
- Gap antar elemen dalam card: 8–12px.
- Gap antar section: 24–32px.
- Jarak antar tombol: 8px.
- Form field spacing: 16px.

---

## 3.4 Border Radius

Gunakan radius semi-rounded agar tampilan terasa modern tetapi tetap profesional.

| Token | Value | Usage |
|---|---:|---|
| `radius-sm` | 6px | Small controls |
| `radius-md` | 8px | Inputs and buttons |
| `radius-lg` | 12px | Cards |
| `radius-xl` | 16px | Large panels and dialogs |
| `radius-full` | 999px | Pills and circular controls |

## 3.5 Borders and Shadows

### Borders

- Gunakan border tipis untuk membedakan panel.
- Hindari penggunaan border pada setiap elemen kecil.
- Gunakan warna border yang sesuai dengan theme.

### Shadows

Gunakan shadow secara minimal.

| Token | Usage |
|---|---|
| Shadow Small | Floating buttons and subtle elements |
| Shadow Medium | Dropdowns and popovers |
| Shadow Large | Dialogs and floating panels |

Pada dark theme, prioritaskan perbedaan surface dan border dibandingkan shadow yang terlalu kuat.

---

## 3.6 Iconography

### Icon Style

- Gunakan icon outline.
- Gunakan gaya ikon yang konsisten.
- Hindari ikon yang terlalu dekoratif.
- Ikon harus mudah dikenali.

### Icon Sizes

| Token | Size | Usage |
|---|---:|---|
| Small | 14px | Inline metadata |
| Default | 16px | Buttons and controls |
| Medium | 20px | Navigation |
| Large | 24px | Empty states and prominent actions |

### Recommended Icon Library

Gunakan ikon Lucide sebagai referensi visual utama.

Contoh ikon:

| Action | Icon Concept |
|---|---|
| Dashboard | Layout Dashboard |
| Downloads | Download |
| Queue | List Ordered |
| History | History |
| Settings | Settings |
| Add Download | Plus |
| Pause | Pause |
| Resume | Play |
| Cancel | X |
| Retry | Refresh Cw |
| Open Folder | Folder Open |
| Completed | Check Circle |
| Failed | Circle Alert |

Implementasi ikon harus menggunakan aset yang kompatibel dengan Slint, misalnya SVG yang telah disiapkan sebagai resource aplikasi.

---

# 4. Layout & Application Structure

## 4.1 Application Layout

SpeedFlow menggunakan pola desktop application dengan sidebar navigation dan content area.

Struktur utama:

```text
+----------------------------------------------------------+
|  SpeedFlow                              Min Max Close     |
+----------------------------------------------------------+
|          |                                               |
| Sidebar  |                 Content Area                  |
|          |                                               |
| Overview |                                               |
| Downloads|                                               |
| Queue    |                                               |
| History  |                                               |
|          |                                               |
|          |                                               |
|          |                                               |
| Settings |                                               |
+----------------------------------------------------------+
```

## 4.2 Main Navigation

Navigasi utama menggunakan sidebar vertikal.

| Navigation | Purpose |
|---|---|
| Overview | Ringkasan aktivitas download |
| Downloads | Semua download |
| Queue | Download yang menunggu |
| History | Riwayat download |
| Settings | Pengaturan aplikasi |

### Navigation Behavior

- Item aktif memiliki background accent yang lembut.
- Ikon dan label ditampilkan bersama.
- Hover state memberikan feedback visual.
- Sidebar tetap konsisten pada halaman utama.
- Navigasi harus dapat diakses melalui keyboard.

## 4.3 Window Layout

### Minimum Window Size

Ukuran minimum awal yang direkomendasikan:

- Width: 900px.
- Height: 600px.

### Recommended Window Size

- Width: 1200px.
- Height: 800px.

### Layout Behavior

- Sidebar memiliki lebar tetap pada ukuran normal.
- Content area menggunakan ruang yang tersedia.
- Pada jendela yang lebih kecil, padding dan kolom informasi dapat disederhanakan.
- Informasi sekunder dapat dipindahkan ke detail view atau menu tambahan.
- Hindari horizontal overflow.

---

# 5. Core UI Components

## 5.1 Buttons

### Button Variants

**Primary Button**

Digunakan untuk tindakan utama seperti Add Download dan Save.

**Secondary Button**

Digunakan untuk tindakan pendukung seperti Cancel atau Edit.

**Ghost Button**

Digunakan untuk tindakan kecil atau kontrol yang tidak membutuhkan penekanan visual besar.

**Danger Button**

Digunakan untuk tindakan destruktif seperti Delete atau Remove.

### Button Sizes

| Size | Height | Horizontal Padding |
|---|---:|---:|
| Small | 30px | 10px |
| Medium | 36px | 14px |
| Large | 42px | 18px |

### Button States

- Default.
- Hover.
- Pressed.
- Focused.
- Disabled.
- Loading.

### Button Rules

- Gunakan label yang jelas.
- Hindari tombol dengan terlalu banyak teks.
- Gunakan ikon jika membantu mengenali tindakan.
- Tombol destructive harus memiliki konfirmasi jika tindakan sulit dibatalkan.

---

## 5.2 Input Fields

### Input Types

- URL Input.
- Text Input.
- Search Input.
- Number Input.
- Folder Path Input.

### Input Structure

```text
Label
+----------------------------------------+
| Placeholder                            |
+----------------------------------------+
Helper text or error message
```

### Input States

- Default.
- Focused.
- Filled.
- Disabled.
- Error.
- Success.

### Input Guidelines

- Label harus selalu tersedia.
- Placeholder hanya menjadi contoh, bukan pengganti label.
- Error message menjelaskan masalah dan cara memperbaikinya.
- URL input harus dapat menerima paste melalui keyboard shortcut.

---

## 5.3 Download Item Card

Download Item Card merupakan salah satu komponen paling penting dalam SpeedFlow.

Komponen ini digunakan untuk menampilkan informasi dan kontrol setiap download.

### Component Structure

```text
+-----------------------------------------------------------+
| [File Icon]  example-file.zip              [Pause] [More] |
|              1.2 GB / 2.0 GB                              |
|              [====================------]  60%            |
|              8.4 MB/s     1m 24s left                     |
+-----------------------------------------------------------+
```

### Required Information

- File icon.
- File name.
- Download status.
- Downloaded size.
- Total file size.
- Progress bar.
- Download speed.
- Estimated time remaining.

### Optional Information

- File type.
- Source domain.
- Download destination.
- Error information.

### Interaction

- Pause: Menghentikan sementara download.
- Resume: Melanjutkan download.
- Cancel: Membatalkan download.
- Retry: Mengulangi download yang gagal.
- More: Membuka tindakan tambahan.

### Component States

| State | Visual Behavior |
|---|---|
| Downloading | Brand Accent progress bar |
| Paused | Warning status |
| Queued | Neutral status |
| Completed | Success indicator |
| Failed | Danger indicator |
| Retrying | Warning indicator with retry feedback |
| Cancelled | Muted appearance |

---

## 5.4 Progress Bar

### Usage

Progress bar digunakan untuk menunjukkan seberapa banyak file yang sudah diunduh.

### Guidelines

- Tinggi default: 6px.
- Border radius: Full.
- Track menggunakan surface secondary.
- Fill menggunakan brand primary.
- Progress harus mengikuti data aktual.
- Jangan menggunakan animasi yang mengesankan progress palsu.

### Progress Types

1. Determinate Progress: Ketika ukuran file diketahui.
2. Indeterminate Progress: Ketika progress belum dapat dihitung.

---

## 5.5 Status Badge

Status Badge digunakan untuk menampilkan kondisi download secara ringkas.

| Status | Label |
|---|---|
| Downloading | Downloading |
| Queued | Queued |
| Paused | Paused |
| Completed | Completed |
| Failed | Failed |
| Retrying | Retrying |
| Cancelled | Cancelled |

Gunakan background lembut dan teks yang memiliki kontras cukup.

---

## 5.6 Data Table

Digunakan pada halaman Downloads, Queue, dan History apabila tampilan daftar lebih sesuai daripada card.

### Recommended Columns

| Column | Description |
|---|---|
| Name | Nama file |
| Size | Ukuran file |
| Status | Status download |
| Progress | Progress download |
| Speed | Kecepatan |
| Added | Waktu ditambahkan |
| Actions | Tindakan |

### Table Guidelines

- Header harus jelas.
- Setiap row memiliki tinggi yang nyaman dibaca.
- Hover memberikan feedback.
- Nama file panjang menggunakan truncation.
- Kolom penting harus tetap terlihat pada ukuran jendela yang lebih kecil.
- Sorting dapat ditambahkan untuk kolom yang relevan.

---

## 5.7 Dialogs

Dialog digunakan untuk tindakan yang membutuhkan perhatian pengguna.

Jenis dialog:

- Add Download Dialog.
- Download Details Dialog.
- Confirm Cancel Dialog.
- Confirm Delete Dialog.
- Error Dialog.

### Dialog Guidelines

- Judul harus menjelaskan konteks.
- Konten harus singkat dan relevan.
- Tombol utama harus mudah ditemukan.
- Tombol Cancel tersedia untuk tindakan yang belum dikonfirmasi.
- Dialog tidak boleh menampilkan informasi yang tidak diperlukan.

---

## 5.8 Dropdown & Context Menu

Digunakan untuk tindakan tambahan yang tidak perlu selalu terlihat.

Contoh:

- Open File.
- Open Folder.
- Copy URL.
- Retry.
- Remove from List.
- Delete Download History.
- Show Details.

Menu harus mendukung keyboard navigation dan memiliki label yang jelas.

---

## 5.9 Toast Notifications

Toast digunakan untuk memberikan feedback singkat tanpa mengganggu aktivitas pengguna.

Contoh:

- Download added to queue.
- Download completed.
- Download paused.
- Download resumed.
- Download failed.
- Settings saved.

### Toast Guidelines

- Posisi default: Bottom Right.
- Durasi tampil harus cukup untuk dibaca.
- Error penting tidak boleh hanya bergantung pada toast sementara.
- Toast tidak boleh menutupi kontrol utama.

---

# 6. Page Design Specifications

## 6.1 Overview Page

### Purpose

Menampilkan ringkasan aktivitas download pengguna.

### Main Elements

1. Page Header.
2. Total Download Speed.
3. Active Downloads Summary.
4. Completed Downloads Summary.
5. Recent Downloads.
6. Quick Actions.

### Layout Concept

```text
+---------------------------------------------------------+
| Overview                                  + Add Download|
|---------------------------------------------------------|
|                                                         |
| +----------------+ +----------------+ +---------------+ |
| | Active         | | Download Speed | | Completed     | |
| | 3              | | 12.4 MB/s      | | 24            | |
| +----------------+ +----------------+ +---------------+ |
|                                                         |
| Recent Downloads                                        |
| +-----------------------------------------------------+ |
| | File A                 Downloading          75%     | |
| | File B                 Completed           100%     | |
| | File C                 Paused               42%     | |
| +-----------------------------------------------------+ |
+---------------------------------------------------------+
```

### Design Rules

- Total download speed menjadi informasi utama.
- Download aktif ditampilkan lebih menonjol.
- Recent Downloads menampilkan aktivitas terbaru.
- Tombol Add Download harus mudah ditemukan.

## 6.2 Downloads Page

### Purpose

Menampilkan semua download yang sedang berjalan maupun yang pernah ditambahkan.

### Main Elements

- Page Header.
- Search.
- Status Filter.
- Download List.
- Download Controls.

### Design Rules

- Informasi progress harus mudah dipindai.
- Status download harus terlihat jelas.
- Pengguna dapat melakukan tindakan tanpa membuka detail.
- Search dan filter harus mudah digunakan.

## 6.3 Queue Page

### Purpose

Menampilkan download yang sedang menunggu giliran.

### Main Elements

- Queue List.
- Queue Position.
- Priority.
- Remove Action.
- Queue Controls.

### Design Rules

- Urutan antrean harus jelas.
- Prioritas harus mudah diubah.
- Informasi jumlah download aktif dan menunggu harus tersedia.

## 6.4 History Page

### Purpose

Menampilkan riwayat aktivitas download.

### Main Elements

- Search.
- Date Filter.
- Status Filter.
- File Name.
- File Size.
- Completion Date.
- File Actions.

### Design Rules

- File yang berhasil diunduh dan gagal harus mudah dibedakan.
- Pengguna dapat membuka file atau folder.
- Riwayat dapat dihapus sesuai kebutuhan.

## 6.5 Settings Page

### Purpose

Mengatur perilaku dan preferensi aplikasi.

### Settings Categories

**General**
- Default download folder.
- Theme selection.
- Application behavior.

**Downloads**
- Default parallel connections.
- Maximum simultaneous downloads.
- Automatic retry.
- Maximum retry attempts.

**Bandwidth**
- Enable bandwidth limit.
- Maximum download speed.

**Notifications**
- Download completed notification.
- Download failed notification.

**Clipboard**
- Enable clipboard monitoring.

### Design Rules

- Pengaturan dikelompokkan berdasarkan kategori.
- Label dan deskripsi harus jelas.
- Perubahan pengaturan harus memiliki feedback.
- Pengaturan lanjutan tidak boleh memenuhi tampilan utama.

---

# 7. Interaction & Motion Design

## 7.1 Interaction Principles

- Respons interaksi harus terasa cepat.
- Setiap tindakan harus memiliki feedback.
- Animasi tidak boleh menghambat fungsi utama.
- Interaksi yang sama harus memiliki perilaku yang konsisten.

## 7.2 Motion Guidelines

| Interaction | Recommended Duration |
|---|---:|
| Button Hover | 120ms |
| Button Press | 80ms |
| Dropdown Open | 140ms |
| Dialog Open | 180ms |
| Sidebar Transition | 180ms |
| Toast Appearance | 180ms |

Gunakan easing yang halus dan hindari animasi berlebihan.

## 7.3 Download Progress Updates

- Progress diperbarui berdasarkan data download engine.
- Kecepatan download menggunakan perhitungan interval yang konsisten.
- ETA diperbarui berdasarkan kecepatan aktual.
- UI tidak perlu menerima update terlalu sering sehingga menyebabkan rendering berlebihan.
- Perubahan status harus langsung terlihat.

## 7.4 Keyboard Shortcuts

Shortcut awal yang direkomendasikan:

| Shortcut | Action |
|---|---|
| `Ctrl + N` | Add Download |
| `Ctrl + V` | Paste URL pada dialog yang aktif |
| `Ctrl + F` | Search |
| `Ctrl + ,` | Open Settings |
| `Delete` | Remove selected item |
| `Space` | Pause/Resume selected download |
| `Escape` | Close dialog or popover |

Shortcut harus disesuaikan dengan konvensi platform target dan tidak boleh mengambil alih shortcut sistem yang penting.

---

# 8. States & Feedback

## 8.1 Loading State

Digunakan ketika aplikasi sedang:

- Mengambil metadata URL.
- Memuat riwayat.
- Menjalankan operasi yang membutuhkan waktu.
- Memproses informasi file.

Gunakan loading indicator yang sederhana dan hindari menampilkan spinner tanpa konteks jika pesan status dapat diberikan.

## 8.2 Empty State

Empty state digunakan ketika belum ada data untuk ditampilkan.

Contoh:

**No Downloads Yet**

Start downloading by adding a file URL.

**Queue Is Empty**

Your queued downloads will appear here.

**No Download History**

Completed and failed downloads will appear here.

Empty state dapat menyediakan tombol tindakan yang relevan.

## 8.3 Error State

Error harus memiliki:

- Judul singkat.
- Penjelasan masalah.
- Tindakan yang dapat dilakukan pengguna.

Contoh:

**Download Failed**

The server could not be reached. Check your connection and try again.

Action: Retry.

## 8.4 Offline State

Ketika jaringan tidak tersedia:

- Download aktif dapat menampilkan status connection error.
- Sistem dapat melakukan retry sesuai konfigurasi.
- Pengguna dapat melakukan retry manual.
- Data progress yang masih valid harus dipertahankan.

## 8.5 Disabled State

Komponen disabled harus memiliki perbedaan visual yang jelas dan tidak memberikan kesan dapat digunakan.

Contoh:
- Resume ketika download masih berjalan.
- Pause ketika download sudah selesai.
- Retry ketika download belum gagal.

---

# 9. Accessibility Guidelines

## 9.1 Keyboard Accessibility

- Seluruh kontrol utama harus dapat diakses dengan keyboard.
- Fokus keyboard harus terlihat jelas.
- Urutan navigasi harus logis.
- Dialog harus mengelola fokus secara tepat.

## 9.2 Visual Accessibility

- Teks harus memiliki kontras yang cukup.
- Jangan menggunakan warna sebagai satu-satunya indikator status.
- Ukuran teks harus nyaman dibaca.
- Komponen interaktif harus memiliki ukuran target yang memadai.

## 9.3 Screen Reader

Jika didukung oleh platform dan implementasi Slint:

- Kontrol harus memiliki accessible label.
- Status download harus dapat dikenali.
- Perubahan status penting harus dapat disampaikan.
- Informasi progress harus memiliki representasi tekstual.

## 9.4 Motion Accessibility

- Animasi harus sederhana.
- Jangan menggunakan animasi berkedip.
- Pergerakan dekoratif yang tidak penting harus dihindari.
- Hormati preferensi pengurangan animasi jika didukung.

---

# 10. Theme System

## 10.1 Theme Modes

SpeedFlow menyediakan tiga pilihan tema:

1. Light.
2. Dark.
3. System.

Default theme adalah System.

## 10.2 Theme Behavior

- System mengikuti preferensi sistem operasi apabila tersedia.
- Light menggunakan palet terang.
- Dark menggunakan palet gelap.
- Pengguna dapat mengganti tema melalui Settings.
- Pilihan tema disimpan secara lokal.

## 10.3 Semantic Token Mapping

Komponen tidak boleh menggunakan warna literal secara sembarangan.

Gunakan semantic token seperti:

- `color-background`
- `color-surface`
- `color-surface-secondary`
- `color-text-primary`
- `color-text-secondary`
- `color-border`
- `color-brand-primary`
- `color-success`
- `color-warning`
- `color-danger`

Dengan sistem ini, komponen dapat mengikuti theme tanpa harus mengubah warna satu per satu.

---

# 11. Design Tokens & Implementation

## 11.1 Token Structure

Seluruh design token harus dikelompokkan secara konsisten.

Contoh struktur:

```text
design-tokens/
├── colors
│   ├── brand
│   ├── neutral
│   └── semantic
├── typography
├── spacing
├── radius
├── shadows
└── components
```

## 11.2 Slint Implementation Guidelines

- Gunakan reusable Slint components.
- Pisahkan UI component dari business logic.
- Hindari duplikasi definisi warna dan ukuran.
- Gunakan semantic token untuk theme.
- Gunakan komponen dengan API yang konsisten.
- State UI harus berasal dari application state yang jelas.
- Jangan menempatkan download engine logic di dalam komponen visual.

## 11.3 Recommended Component Structure

```text
src/
├── ui/
│   ├── app-window.slint
│   ├── components/
│   │   ├── button.slint
│   │   ├── input.slint
│   │   ├── icon-button.slint
│   │   ├── progress-bar.slint
│   │   ├── status-badge.slint
│   │   ├── download-item.slint
│   │   ├── dialog.slint
│   │   └── toast.slint
│   │
│   ├── layouts/
│   │   ├── main-layout.slint
│   │   └── sidebar.slint
│   │
│   ├── pages/
│   │   ├── overview.slint
│   │   ├── downloads.slint
│   │   ├── queue.slint
│   │   ├── history.slint
│   │   └── settings.slint
│   │
│   └── theme/
│       ├── colors.slint
│       ├── typography.slint
│       └── tokens.slint
│
├── application/
├── download/
├── storage/
└── main.rs
```

Struktur tersebut merupakan rekomendasi awal dan dapat disesuaikan dengan struktur project Rust yang sebenarnya.

## 11.4 Component Reusability

Komponen harus dapat digunakan kembali di berbagai halaman.

Contoh:

- Button digunakan pada seluruh halaman.
- Status Badge digunakan pada Downloads, Queue, dan History.
- Download Item digunakan pada Overview dan Downloads.
- Dialog digunakan untuk Add Download dan konfirmasi tindakan.
- Toast digunakan untuk feedback global.

---

# 12. Design Quality Assurance

## 12.1 Visual Consistency Checklist

- [ ] Seluruh halaman menggunakan semantic color tokens.
- [ ] Typography mengikuti skala yang ditentukan.
- [ ] Spacing konsisten.
- [ ] Border radius konsisten.
- [ ] Ikon memiliki gaya visual yang sama.
- [ ] Button memiliki state yang lengkap.
- [ ] Input memiliki state yang lengkap.
- [ ] Download status memiliki representasi yang konsisten.
- [ ] Light dan Dark Theme memiliki kontras yang baik.

## 12.2 Functional UI Checklist

- [ ] Semua tombol memiliki fungsi yang sesuai.
- [ ] Navigasi antarhalaman berjalan.
- [ ] Dialog dapat dibuka dan ditutup.
- [ ] Input dapat menerima data.
- [ ] Progress bar menampilkan nilai aktual.
- [ ] Status Badge mengikuti status download.
- [ ] Empty State ditampilkan ketika tidak ada data.
- [ ] Error State memberikan informasi yang jelas.

## 12.3 Accessibility Checklist

- [ ] Kontrol utama dapat digunakan dengan keyboard.
- [ ] Fokus terlihat jelas.
- [ ] Teks memiliki kontras memadai.
- [ ] Status tidak hanya dibedakan melalui warna.
- [ ] Label komponen jelas.
- [ ] Navigasi memiliki urutan logis.

---

# 13. Design Constraints & Decisions

## 13.1 Technical Constraints

- Framework UI utama adalah Slint.
- Aplikasi menggunakan Rust sebagai bahasa utama.
- Desain ditujukan untuk desktop, bukan mobile.
- Komponen harus mempertimbangkan kemampuan dan batasan Slint.
- Animasi harus tetap ringan.
- UI tidak boleh menghambat proses download yang berjalan asynchronous.

## 13.2 Design Decisions

| Decision | Value |
|---|---|
| Design Direction | Modern Minimalist |
| Brand Accent | Sage Green (`#78C375`) |
| Primary Typeface | Inter |
| Icon Style | Outline |
| Main Layout | Sidebar + Content |
| Theme | Light, Dark, System |
| Default Theme | System |
| Spacing Base | 4px |
| Default Card Radius | 12px |
| Default Button Height | 36px |
| Minimum Window Size | 900 × 600px |

## 13.3 Open Questions

- [x] Identitas visual aksen menggunakan warna hijau `#78C375`.
- [ ] Apakah sidebar akan selalu expanded atau dapat di-collapse?
- [ ] Apakah Overview perlu menampilkan grafik kecepatan download secara langsung?
- [ ] Apakah Downloads lebih cocok menggunakan card list atau data table sebagai tampilan default?
- [ ] Apakah perlu tampilan compact mode untuk pengguna yang mengelola banyak download?
- [ ] Apakah target OS pertama membutuhkan penyesuaian visual khusus?

---

# 14. Appendix

## 14.1 References

- **SRS:** Software Requirements Specification — SpeedFlow v0.1.
- **PRD:** Product Requirements Document — SpeedFlow v0.1.
- **Slint Documentation:** https://slint.dev/
- **Lucide Icons:** https://lucide.dev/
- **Inter Typeface:** https://fonts.google.com/specimen/Inter

## 14.2 Changelog

| Date | Version | Changes | Author |
|---|---|---|---|
| 2026-11-03 | 0.1 | Initial Design System draft | Product/Design |

## 14.3 Approval

- [ ] Product Owner — TBD.
- [ ] Engineering Lead — TBD.
- [ ] UI/UX Designer — TBD.

---

**End of Document — SpeedFlow Design System v0.1**