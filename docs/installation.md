# Installation & Setup

Set up and configure the various modules of the **Parlez-vous?** ecosystem.

---

## 1. Desktop & Android Client Compilation

The client application compiles as a desktop application (Linux/Windows/macOS) or an Android APK.

### Prerequisites:
1. **Rust & Cargo**: Install from [rustup.rs](https://rustup.rs).
2. **Node.js & pnpm**: Install Node.js and run `npm install -g pnpm`.
3. **System Dependencies (Linux)**:
   ```bash
   sudo apt install libglib2.0-dev libgtk-3-dev libjavascriptcoregtk-4.1-dev libsoup-3.0-dev libwebkitgtk-6.0-dev libssl-dev libwebkit2gtk-4.1-dev
   ```

### Local Build:
1. Navigate to the client directory:
   ```bash
   cd parlezvous
   pnpm install
   ```
2. Build the Jamo handwriting model (detailed in [Development](file:///home/user/Documents/lang/parlezvous/docs/development.md)) and copy `character_model.onnx` and `character_model.onnx.data` into `parlezvous/src-tauri/models/`.
3. Run the desktop application in development mode:
   ```bash
   pnpm tauri dev
   ```

### Android APK Build (Optional):
1. Make sure Android Studio, SDK, and Kotlin are installed, export paths, then build:
   ```bash
   export KOTLIN_HOME=/home/user/.sdkman/candidates/kotlin/2.3.21
   export KOTLIN=/home/user/.sdkman/candidates/kotlin/2.3.21/bin/kotlin
   export JAVA_HOME=/home/user/android-studio/jbr/
   export ANDROID_HOME=/home/user/Android/Sdk
   export NDK=/home/user/Android/Sdk/ndk/30.0.14904198
   export JAVA=/usr/bin/java
   ```

2. **To generate a Signed Release APK (Required for distribution):**
   By default, `pnpm tauri android build` attempts to build a release APK. To sign it correctly, you must generate a keystore and configure a properties file:
   
   First, generate your keystore file:
   ```bash
   keytool -genkey -v -keystore ~/upload-keystore.jks -keyalg RSA -keysize 2048 -validity 10000 -alias upload
   ```
   
   Then, create a file at `parlezvous/src-tauri/gen/android/keystore.properties` containing your passwords and path:
   ```properties
   password=your_keystore_password
   keyAlias=upload
   storeFile=/home/user/upload-keystore.jks
   ```
   *(Note: `keystore.properties` is already in `.gitignore` to prevent leaking your credentials).*

3. Build the APK package:
   ```bash
   pnpm tauri android build --apk
   ```

---

## 2. Speech-to-Text (Whisper ASR)

The Voice Chat feature requires a speech-to-text transcription server. A self-hosted OpenAI-compatible Whisper container is configured to handle voice inputs:

1. Navigate to the `whisper` directory:
   ```bash
   cd whisper
   ```
2. Configure GPU acceleration in `docker-compose.yml` if you have an NVIDIA GPU (highly recommended for performance).
3. Build and launch the container:
   ```bash
   docker compose up --build -d
   ```
This exposes the ASR API endpoint at `http://localhost:8000/v1/audio/transcriptions`.

### Configuring Model Size:
By default, the server loads the `"tiny"` model size for fast, low-memory inference. To use a different model size (such as `"base"`, `"small"`, `"medium"`, or `"large"`):
1. Open the [app.py](file:///home/user/Documents/lang/parlezvous/whisper/app.py) configuration file.
2. Locate the model loading block and change `model_name`:
   ```python
   # --- Model Loading ---
   model_name = "base"  # Options: tiny, base, small, medium, large
   ```
3. Rebuild and restart the container to download and load the new model size:
   ```bash
   docker compose up --build -d
   ```

### Platform Configurations:
* **Desktop Client:** Connects to the local container at `http://localhost:8000/v1/audio/transcriptions` (default).
* **Mobile Client (Android APK):** Because a physical mobile device cannot access your host machine's `localhost`, you must configure the ASR Server URL in the mobile application settings to point to the local network IP of your host machine running the Whisper container (e.g., `http://192.168.1.50:8000/v1/audio/transcriptions`).

---

## 3. Text-to-Speech (TTS) & Voice Output

Parlez-vous? supports two modes for generating vocal output for the 3D avatar, depending on whether you want server-hosted or fully on-device operations:

### Option A: Server-Based TTS using Qwen3-TTS (Best for Desktop / High Quality)
1. Navigate to the `Qwen3-TTS` directory:
   ```bash
   cd Qwen3-TTS
   ```
2. Install dependencies using the `uv` tool:
   ```bash
   uv sync
   ```
3. Run the OpenAI-compatible TTS server with the custom voice checkpoint:
   ```bash
   uv run qwen_tts/cli/openai_server.py --port 5050 --checkpoint Qwen3-TTS-12Hz-0.6B-CustomVoice/ --no-flash-attn
   ```
This exposes the TTS server on port `5050` at the endpoint `http://localhost:5050/v1/audio/speech`.

* **Desktop Setup:** Configured to point directly to `http://localhost:5050/v1/audio/speech` (default).
* **Mobile Setup:** To use the server, update the TTS Server URL in the mobile app settings to your host machine's local network IP (e.g., `http://192.168.1.50:5050/v1/audio/speech`).

### Option B: Local On-Device TTS using Supertonic (Best for Mobile / Offline)
To support offline operation on mobile devices without running a Python server:
1. The Android application includes the **Supertonic TTS** plugin (powered by `onnxruntime` and Kotlin).
2. Go to the Settings screen in the application.
3. Download the `supertonic-3` models directly. Once downloaded, the application will run all TTS synthesis locally on the device, eliminating the need for an external TTS server.

---

## 4. Local Large Language Models (LLM)

The conversational tutor and journaling grading engine can run on local computers or on-device on mobile:

### Option A: Server-Based LLM using Ollama (Desktop & local-network Mobile)
1. Install Ollama from [ollama.ai](https://ollama.ai).
2. Run your preferred model (we recommend `gemma-4-E2B-it` or similar capability instruct model):
   ```bash
   ollama run gemma-4-E2B-it
   ```
* **Desktop Setup:** Connects automatically to `http://localhost:11434` (default).
* **Mobile Setup:** You can connect the mobile client to the Ollama server running on your host computer by updating the Ollama Server URL in the settings to your local network IP (e.g., `http://192.168.1.50:11434`). Make sure to launch Ollama with the environment variable `OLLAMA_HOST=0.0.0.0` so it accepts network connections.

### Option B: Local On-Device LLM using Google LiteRT (Mobile / Offline)
To support fully offline, private chat and journaling on mobile devices:
1. The Android client features the **Google LiteRT** (TensorFlow Lite) plugin.
2. Enter your HuggingFace Access Token in the app settings panel.
3. Use the in-app download buttons to download the `gemma-4-E2B-it.litertlm` model file and tokenizer directly to your phone.
4. The application will automatically route all chat and journal prompts to run locally on the mobile phone's CPU/GPU/NPU.
