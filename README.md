# Parlez-vous? 💬

Language learning is a wonderful way to expand the reachable cities of our cognitive highways, so when some fall out of repair you can get to your destination. Learn a language today using AI supports! Live chat encourages active production helping you converse. Privacy focused language learning has never been easier!
---

## 🌐 Project Links & Sites
* **Official Website:** [parlezvous.ca](https://parlezvous.ca)

---

## 🛠️ Built With

![Tauri](https://img.shields.io/badge/Tauri-%2324C8DB?style=for-the-badge&logo=tauri&logoColor=white)
![SvelteKit](https://img.shields.io/badge/Svelte-%23FF3E00?style=for-the-badge&logo=svelte&logoColor=white)
![Rust](https://img.shields.io/badge/Rust-black?style=for-the-badge&logo=rust&logoColor=white)
![Kotlin](https://img.shields.io/badge/Kotlin-%237F52FF?style=for-the-badge&logo=kotlin&logoColor=white)
![TypeScript](https://img.shields.io/badge/TypeScript-%23007ACC?style=for-the-badge&logo=typescript&logoColor=white)
![Android](https://img.shields.io/badge/Android-%233DDC84?style=for-the-badge&logo=android&logoColor=white)
![PyTorch](https://img.shields.io/badge/PyTorch-%23EE4C2C?style=for-the-badge&logo=pytorch&logoColor=white)
![ONNX](https://img.shields.io/badge/ONNX-%23005C99?style=for-the-badge&logo=onnx&logoColor=white)
![SQLite](https://img.shields.io/badge/SQLite-%2307405E?style=for-the-badge&logo=sqlite&logoColor=white)

---

## 🚀 Quick Start Installation

For full deployment instructions, please reference the [Installation Guide](file:///home/user/Documents/lang/parlezvous/docs/installation.md). Below is a quick overview:

Download APK in the releases tab for a quick mobile start. 

### 1. Prerequisites
- Clone the repository
- Install [uv](https://github.com/astral-sh/uv) (Python packaging)
- Install [Ollama](https://ollama.com/) (Local AI inference)

### 2. HangulNist (Handwriting Recognition Model)
1. Build the Jamo recognition model:
   ```bash
   uv sync && uv run main.py
   ```
2. Copy the built model into the Tauri source:
   ```bash
   cp character_model.onnx.data ../parlezvous/src-tauri/models/
   cp character_model.onnx ../parlezvous/src-tauri/models/
   ```

### 3. Parlez-vous App Setup
1. Install [pnpm](https://pnpm.io/) and [Rust](https://www.rust-lang.org/)
2. Install UI dependencies:
   ```bash
   pnpm i
   ```
3. *(Linux only)* You may need system libraries:
   ```bash
   sudo apt install libglib2.0-dev libgtk-3-dev libjavascriptcoregtk-4.1-dev libsoup-3.0-dev libwebkitgtk-6.0-dev libssl-dev libwebkit2gtk-4.1-dev
   ```

### 4. Android Compilation (Optional)
Make sure Android Studio, SDK, and Kotlin are installed, export paths, then build:

Before building, you must fetch the proprietary device runtimes:
```bash
./parlezvous/libraries/fetch_qualcomm_library.sh
```

```bash
export KOTLIN_HOME=/home/user/.sdkman/candidates/kotlin/2.3.21
export KOTLIN=/home/user/.sdkman/candidates/kotlin/2.3.21/bin/kotlin
export JAVA_HOME=/home/user/android-studio/jbr/
export ANDROID_HOME=/home/user/Android/Sdk
export NDK=/home/user/Android/Sdk/ndk/30.0.14904198
export JAVA=/usr/bin/java

pnpm tauri android build
```

### 5. Running AI Helpers
* **Whisper ASR (Speech-to-Text):** Run `docker compose up` inside the `whisper/` directory.
* **Qwen3 TTS (Text-to-Speech):** 
  ```bash
  uv run qwen_tts/cli/openai_server.py --port 5050 --checkpoint Qwen3-TTS-12Hz-0.6B-CustomVoice/ --no-flash-attn
  ```

---

## 🔒 Post Installation Firewall
Allow connection from desired IP addresses to the specific ports:
```bash
sudo ufw allow from <IP_ADDRESS> proto tcp to any port <PORT_NUMBER>
```

---

# License

This project is licensed under the [MIT License](LICENSE).

## Models, Assets & Licenses

Parlez-vous integrates external machine learning models and 3D assets, which are subject to their own respective licensing terms. Users and developers of this application must comply with these terms:

1. **Gemma 4** (`gemma-4-E2B-it`):
   - **License**: [Apache License 2.0](https://choosealicense.com/licenses/apache-2.0/)
   - A permissive open-source license that allows commercial use, modification, distribution, and patent grants.

2. **Supertonic TTS** (`supertonic-3`):
   - **License**: [BigScience Open RAIL-M License](https://huggingface.co/spaces/bigscience/license)
   - A responsible AI license designed for open access while enforcing ethical downstream use restrictions.
   - **Important Use Restrictions**: The model must not be used for unlawful acts, harm to minors, generating disinformation, harassment/impersonation, automated legal decision-making, social scoring or discrimination, medical advice/interpretation, or certain law enforcement activities. Any redistribution or derivation of the model weights must carry these same use-based restrictions.

3. **3D Assets** (`avatar.vrm`, `VRMA_*.vrma`):
   - **License**: Proprietary terms dictated by pixiv Inc.'s VRoid Project.
   - You must include the following attribution in derivatives: "キャラクターアニメーション: ピクシブ株式会社 VRoidプロジェクト" (Character animation credits to pixiv Inc.'s VRoid Project).
   - Commercial use is allowed with credit. See VRoid Hub for full terms.
