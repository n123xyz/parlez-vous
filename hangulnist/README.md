# HangulNist

This directory contains a handwriting recognition model for Hangul characters (specifically Jamo). 

## Model and Dataset

The model (`character_model.onnx.data`) was trained on the **Handwritten Hangul Characters** dataset by jamescasia, originally sourced from Kaggle:
[https://www.kaggle.com/datasets/jamescasia/handwritten-hangul-characters](https://www.kaggle.com/datasets/jamescasia/handwritten-hangul-characters)

## License

Because the model was trained on the aforementioned dataset, which is licensed under the **GNU General Public License v2.0 (GPL-2.0)**, the resulting model weights and the code in this directory are distributed under the same GPL-2.0 license. 

See the [LICENSE](LICENSE) file for the full text of the GPL-2.0 license.

## Cyrillic & Multi-Alphabet Training (Russian & Ukrainian)

To train the Cyrillic handwriting recognition model on Kaggle datasets (`olgabelitskaya/handwritten-russian-letters` and `gregvial/comnist`), use `uv`:

```bash
# Install dependencies
uv sync

# Train Cyrillic model and export to ONNX (auto-copies to parlezvous/src-tauri/models/)
uv run train_cyrillic.py

# Optional arguments:
uv run train_cyrillic.py --dataset russian --epochs 30
uv run train_cyrillic.py --dataset comnist
```