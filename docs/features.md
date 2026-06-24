# Application Features

**Parlez-vous?** includes a suite of interactive tools designed to implement active, output-oriented language acquisition.

---

## 1. Guided Journaling & Grading

The Journal feature encourages structured daily writing practice:

- **Prompt Generation**: The system generates a personalized writing prompt based on the user's selected language, CEFR tier (A1-C2), active theme (e.g. food, travel), and inputs like current weather, activity, and mood.
- **Grading & Feedback**: Once written, the LLM grades the entry. It provides corrections, explanations for grammar errors, and alternative phrasing.
- **Vocabulary Extraction**: The grading engine automatically parses vocabulary words used in the entry and extracts them into native-target pairs, inserting them into the user's database.

---

## 2. Interactive 3D Avatar & Lip Sync

To simulate conversational speaking practice, the chat screen features a fully interactive 3D virtual avatar:

- **VRM Models**: Renders sample VRM avatars using `Three.js` and the `@pixiv/three-vrm` library.
- **Animations**: Loads VRMA animations dynamically (e.g. idling, talking, nodding).
- **Lip Syncing**: Uses the Web Audio API to create a lip-sync analyser node. Audio output from the Text-to-Speech (TTS) engine is processed to map volume frequencies directly to the avatar's mouth blend shapes (specifically the `vrm.expression.setValue('aa', value)` parameter), creating realistic real-time talking animations.
- **Voice Input (VAD + ASR)**: Users can speak to the avatar. Voice Activity Detection (VAD) monitors mic signals, triggering Whisper speech-to-text to transcribe the audio and feed it into the chat session.

---

## 3. Spaced Repetition System (SRS)

A custom Spaced Repetition System helps commit new words and characters to long-term memory:

- **Algorithm**: Decoupled SRS scheduler calculating intervals using custom ease factors and review levels.
- **Inference Integration**: Successfully completing a handwriting test or flashcard updates the `next_review_date` and increases interval multipliers. Failing resets the level to ensure immediate re-evaluation.

---

## 4. Hangul Handwriting Recognition Canvas

For languages with phonetic characters like Korean (Hangul), the app includes a drawing canvas:

- **Interactive Handwriting**: Draw characters directly on a touch/mouse canvas.
- **CNN Classification**: When a character is completed, the canvas pixel buffer is sent to a custom convolutional neural network (CNN) model built with PyTorch and exported as an ONNX runtime model.
- **Burn Integration**: The Tauri backend runs inference on the stroke pixels using the ONNX model, evaluating character matches (e.g. initial, vowel, and final Jamo parts) to verify if the user has correctly written the requested character.
