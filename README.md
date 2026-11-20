<div align="center">

<img src="crates/app/assets/logo.svg" alt="DocumentKit logo" width="112" height="112"/>

# DocumentKit

### Every PDF tool you need. One fast, private, open-source app.

Organize · Optimize · Convert · Edit · Secure · Analyze — all locally, powered by Rust.

<br/>

![Windows](https://img.shields.io/badge/Windows-0078D4?style=for-the-badge&logo=windows11&logoColor=white)
![macOS](https://img.shields.io/badge/macOS-000000?style=for-the-badge&logo=apple&logoColor=white)
![Linux](https://img.shields.io/badge/Linux-FCC624?style=for-the-badge&logo=linux&logoColor=black)
![iOS](https://img.shields.io/badge/iOS-147EFB?style=for-the-badge&logo=apple&logoColor=white)
![Android](https://img.shields.io/badge/Android-3DDC84?style=for-the-badge&logo=android&logoColor=white)

![Rust](https://img.shields.io/badge/Rust-000000?style=flat-square&logo=rust&logoColor=white)
![Dioxus](https://img.shields.io/badge/Dioxus-0F172A?style=flat-square)
![SQLite](https://img.shields.io/badge/SQLite-003B57?style=flat-square&logo=sqlite&logoColor=white)
![License](https://img.shields.io/badge/license-GPLv3-green?style=flat-square)
![Status](https://img.shields.io/badge/status-in%20development-yellow?style=flat-square)

[Features](#-features) · [Roadmap](#%EF%B8%8F-roadmap) · [Team](#-team) · [Contributing](#-contributing) · [License](#-license)

</div>

---

> [!NOTE]
> DocumentKit is in **early development**. Everything below is planned — follow progress in the [roadmap](#%EF%B8%8F-roadmap). A web application is coming soon.

## ✨ Why DocumentKit?

| 🔒 Private | ⚡ Fast | 🖥️ Cross-platform | 🆓 Open source |
| :--- | :--- | :--- | :--- |
| Documents are processed locally. | Native Rust core, no bloat. | Desktop and mobile. | GPLv3, free forever. |

## 🧰 Features

<table>
<tr>
<td width="50%" valign="top">

### 🗂️ Organize
- [ ] Merge PDF
- [ ] Split PDF
- [ ] Remove pages
- [ ] Extract pages
- [ ] Organize PDF
- [ ] Scan to PDF

</td>
<td width="50%" valign="top">

### 🚀 Optimize
- [ ] Compress PDF
- [ ] Repair PDF
- [ ] OCR PDF

</td>
</tr>
<tr>
<td valign="top">

### 📥 Convert to PDF
- [ ] JPG to PDF
- [ ] Word to PDF
- [ ] PowerPoint to PDF
- [ ] Excel to PDF
- [ ] HTML to PDF

</td>
<td valign="top">

### 📤 Convert from PDF
- [ ] PDF to JPG
- [ ] PDF to Word
- [ ] PDF to PowerPoint
- [ ] PDF to Excel
- [ ] PDF to PDF/A

</td>
</tr>
<tr>
<td valign="top">

### ✏️ Edit
- [ ] Rotate PDF
- [ ] Add page numbers
- [ ] Add watermark
- [ ] Crop PDF
- [ ] Edit PDF
- [ ] PDF forms

</td>
<td valign="top">

### 🛡️ Security
- [ ] Unlock PDF
- [ ] Protect PDF
- [ ] Sign PDF
- [ ] Redact PDF
- [ ] Compare PDF

</td>
</tr>
<tr>
<td valign="top">

### 🧠 Intelligence
- [ ] AI summarizer
- [ ] Translate PDF
- [ ] PDF to Markdown

</td>
<td valign="top">

### 📷 Scan
- [ ] Document scanner integration
- [ ] Auto crop & deskew

</td>
</tr>
</table>

## 🏗️ Tech Stack

| Layer | Technology |
| :--- | :--- |
| Core engine | **Rust** |
| UI framework | **Dioxus** |
| Local storage | **SQLite** |
| Targets | Windows · macOS · Linux · iOS · Android · Web (soon) |

## 🗺️ Roadmap

### Platforms
- [ ] 🪟 Windows
- [ ] 🍎 macOS
- [ ] 🐧 Linux
- [ ] 📱 iOS
- [ ] 🤖 Android
- [ ] 🌐 Web application

### Task allocation

| Category | Tools | Assignee | Status |
| :--- | :--- | :--- | :---: |
| Organize | Merge PDF, Split PDF, Remove pages, Extract pages | Shafat | ⏳ Pending |
| Organize | Organize PDF, Scan to PDF | Nahin | ⏳ Pending |
| Optimize | Compress PDF, Repair PDF, OCR PDF | _Unassigned_ | ⏳ Pending |
| Convert To | JPG to PDF, Word to PDF, HTML to PDF | _Unassigned_ | ⏳ Pending |
| Convert To | PowerPoint to PDF, Excel to PDF | _Unassigned_ | ⏳ Pending |
| Convert From | PDF to JPG, PDF to Word, PDF to PDF/A | _Unassigned_ | ⏳ Pending |
| Convert From | PDF to PowerPoint, PDF to Excel | _Unassigned_ | ⏳ Pending |
| Edit | Rotate PDF, Add page numbers, Add watermark | _Unassigned_ | ⏳ Pending |
| Edit | Crop PDF, Edit PDF, PDF forms | _Unassigned_ | ⏳ Pending |
| Security | Unlock PDF, Protect PDF, Sign PDF | _Unassigned_ | ⏳ Pending |
| Security | Redact PDF, Compare PDF | _Unassigned_ | ⏳ Pending |
| Intelligence | AI summarizer, Translate PDF, PDF to Markdown | _Unassigned_ | ⏳ Pending |
| Scan | Document scanner integration, Auto crop & deskew | _Unassigned_ | ⏳ Pending |

## 👥 Team

| Name | Role |
| :--- | :--- |
| **Shafat** | Lead Developer |
| **Nahin** | Core Coding |
| **Joynur** | UI / UX |
| **Samin** | QA Testing |

## 🚀 Build from source

```sh
git clone https://github.com/Shafat21/DocumentKit.git
cd DocumentKit
cargo run -p documentkit      # desktop app
cargo test --workspace        # all tests
```

Linux needs WebKitGTK first: `sudo apt install libwebkit2gtk-4.1-dev libxdo-dev libssl-dev`. For installers, mobile builds and the full setup, see [CONTRIBUTING.md](CONTRIBUTING.md).

```
crates/core/              dk-core: tool contract, catalog, SQLite store, theme
crates/modules/<module>/  one crate per feature module, one file per tool
crates/app/               Dioxus app (desktop + mobile)
docs/                     plan, brainstorm, architecture decisions
```

## 🤝 Contributing

Contributions are welcome! Read [CONTRIBUTING.md](CONTRIBUTING.md) for setup, the code standards, and how to implement a tool. For the architecture, see [docs/PLAN.md](docs/PLAN.md).

1. Open an issue to report a bug or propose a feature.
2. For larger changes, discuss the approach in the issue first.
3. Fork, branch (`feat/<module>/<tool>`), and open a pull request.

## 📜 License

Released under the [GNU General Public License v3.0](LICENSE).

<div align="center">

<sub>Built with 🦀 Rust</sub>

</div>
