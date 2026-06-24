# Developer & Contribution Guide

Instructions on how to extend the codebase, train custom models, and modify curriculum parameters.

---

## 1. Training & Exporting the Jamo CNN Model

The handwriting canvas utilizes a custom-trained CNN classification model built in PyTorch.

1. **Prerequisites**: Make sure you have `uv` installed. Navigate to the `hangulnist` folder:
   ```bash
   cd hangulnist
   ```
2. **Setup**: Install python dependencies:
   ```bash
   uv sync
   ```
3. **Training pipeline**:
   - `main.py` is configured to download the **Wayperwayp Hangul Characters Dataset** from Kaggle Hub.
   - It trains a 3-layer Convolutional Neural Network with Batch Normalization and Focal Loss (to focus on hard-to-distinguish stroke differences).
   - Images are resized to `28x28`, inverted, and threshold-sharpened.
   - To trigger the training:
     ```bash
     uv run main.py
     ```
4. **ONNX Export**:
   - The training script automatically compiles and exports the trained PyTorch network weights into an ONNX model file named `character_model.onnx` and its weight data file `character_model.onnx.data`.
5. **Distribution**: Copy these generated files into the Tauri source tree to make them accessible to the Rust inference backend:
   ```bash
   cp character_model.onnx character_model.onnx.data ../parlezvous/src-tauri/models/
   ```

---

## 2. Modifying the Curriculum Tiers & Themes

The learning structure is defined inside `parlezvous/src/lib/curriculum.ts`:

- The curriculum maps levels 1 to 7 corresponding to CEFR levels from **A1** (The Outskirts) to **Mastery/C2** (The Horizon).
- Each tier contains a specific set of thematic modules (e.g. `greetings`, `numbers`, `family`, `shopping`, `travel`, `science`).
- To add a new theme:
  1. Add a new object inside the `themes` array of the appropriate `Tier` in `CURRICULUM_TIERS`.
  2. Define a unique `id`, a friendly `name`, and a detailed `description`.
  3. The LLM generation services will automatically query the theme database and use the description as semantic seeds in prompts.

---

## 3. Database Migrations

SQLite schema updates are managed inside `parlezvous/src-tauri/src/db/mod.rs`:

- To alter tables or add columns, define a new schema SQL batch constant (e.g. `pub const SCHEMA_V17: &str = ...`).
- Increment the global constant `const DB_VERSION_NUM: usize` by 1.
- Append your new constant `SCHEMA_V17` to the `schemas` array inside `init_db`.
- The migrator automatically compares the database's `PRAGMA user_version` value and applies all pending schema migrations sequentially when the application boots up.
