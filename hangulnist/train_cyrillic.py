"""
Handwritten Cyrillic Alphabet Model Training Pipeline
---------------------------------------------------
Supports:
1. Handwritten Russian Letters (olgabelitskaya/handwritten-russian-letters)
2. CoMNIST - Cyrillic-oriented MNIST (gregvial/comnist)
3. Ukrainian Cyrillic extension (є, і, ї, ґ)

Usage with uv:
    uv run train_cyrillic.py
    uv run train_cyrillic.py --dataset russian --epochs 25
    uv run train_cyrillic.py --dataset comnist --copy-to-tauri
"""

import os
import sys
import json
import shutil
import argparse
from pathlib import Path
from typing import List, Tuple, Dict, Optional

import torch
import torch.nn as nn
import torch.nn.functional as F
import torch.optim as optim
from torch.utils.data import Dataset, DataLoader, random_split, ConcatDataset
from torchvision import transforms
from PIL import Image
import numpy as np
import kagglehub

# Standard Russian Cyrillic Alphabet (33 letters)
RUSSIAN_CYRILLIC_LETTERS = [
    "а", "б", "в", "г", "д", "е", "ё", "ж", "з", "и", "й",
    "к", "л", "м", "н", "о", "п", "р", "с", "т", "у", "ф",
    "х", "ц", "ч", "ш", "щ", "ъ", "ы", "ь", "э", "ю", "я"
]

# Ukrainian-unique letters
UKRAINIAN_EXTRA_LETTERS = ["є", "і", "ї", "ґ"]

# Full Cyrillic alphabet supported (33 Russian + 4 Ukrainian unique = 37 letters)
ALL_CYRILLIC_LETTERS = sorted(list(set(RUSSIAN_CYRILLIC_LETTERS + UKRAINIAN_EXTRA_LETTERS)))


class FocalLoss(nn.Module):
    def __init__(self, gamma: float = 2.0):
        super().__init__()
        self.gamma = gamma

    def forward(self, inputs: torch.Tensor, targets: torch.Tensor) -> torch.Tensor:
        ce_loss = F.cross_entropy(inputs, targets, reduction="none")
        pt = torch.exp(-ce_loss)
        focal_loss = ((1 - pt) ** self.gamma) * ce_loss
        return focal_loss.mean()


class AlphabetNet(nn.Module):
    """
    3-layer CNN with BatchNorm matching the HangulNet architecture
    Input: [Batch, 1, 28, 28]
    """
    def __init__(self, num_classes: int):
        super().__init__()
        self.features = nn.Sequential(
            # Block 1
            nn.Conv2d(1, 32, kernel_size=3, padding=1),
            nn.BatchNorm2d(32),
            nn.ReLU(),
            nn.Conv2d(32, 32, kernel_size=3, padding=1),
            nn.BatchNorm2d(32),
            nn.ReLU(),
            nn.MaxPool2d(2),  # [32, 14, 14]

            # Block 2
            nn.Conv2d(32, 64, kernel_size=3, padding=1),
            nn.BatchNorm2d(64),
            nn.ReLU(),
            nn.MaxPool2d(2),  # [64, 7, 7]

            # Block 3
            nn.Conv2d(64, 128, kernel_size=3, padding=1),
            nn.BatchNorm2d(128),
            nn.ReLU(),
        )

        self.classifier = nn.Sequential(
            nn.Flatten(),
            nn.Linear(128 * 7 * 7, 256),
            nn.BatchNorm1d(256),
            nn.ReLU(),
            nn.Dropout(0.3),
            nn.Linear(256, num_classes),
        )

    def forward(self, x: torch.Tensor) -> torch.Tensor:
        x = self.features(x)
        x = self.classifier(x)
        return x


class RussianLettersDataset(Dataset):
    """
    Loads samples from olgabelitskaya/handwritten-russian-letters.
    Handles CSV metadata mapping image files to letter labels.
    """
    def __init__(self, root_dir: str, class_to_idx: Dict[str, int], transform=None):
        self.root_dir = Path(root_dir)
        self.class_to_idx = class_to_idx
        self.transform = transform
        self.samples: List[Tuple[Path, int]] = []

        # Look for CSV files in dataset
        csv_files = list(self.root_dir.glob("*.csv")) + list(self.root_dir.glob("**/*.csv"))
        
        found_from_csv = False
        if csv_files:
            try:
                import pandas as pd
                for csv_path in csv_files:
                    df = pd.read_csv(csv_path)
                    col_file = next((c for c in df.columns if "file" in c.lower() or "image" in c.lower()), None)
                    col_char = next((c for c in df.columns if "letter" in c.lower() or "label" in c.lower() or "char" in c.lower()), None)
                    
                    if col_file and col_char:
                        parent_dir = csv_path.parent
                        for _, row in df.iterrows():
                            fn = str(row[col_file])
                            img_path = parent_dir / fn
                            if not img_path.exists():
                                matches = list(self.root_dir.glob(f"**/{fn}"))
                                if matches:
                                    img_path = matches[0]
                                else:
                                    continue
                            
                            char_val = str(row[col_char]).strip().lower()
                            if char_val.isdigit():
                                idx = int(char_val) - 1
                                if 0 <= idx < len(RUSSIAN_CYRILLIC_LETTERS):
                                    char_val = RUSSIAN_CYRILLIC_LETTERS[idx]
                            
                            if char_val in self.class_to_idx:
                                self.samples.append((img_path, self.class_to_idx[char_val]))
                                found_from_csv = True
            except Exception as e:
                print(f"[!] Warning reading Russian letters CSV: {e}")

        # Folder and filename pattern scanning
        if not found_from_csv:
            for ext in ("*.png", "*.jpg", "*.jpeg"):
                for img_path in self.root_dir.glob(f"**/{ext}"):
                    name = img_path.stem.lower()
                    folder_name = img_path.parent.name.lower()
                    char_val = None

                    # 1. Check XX_YY_ZZ pattern (e.g. 00_00_00_0000.png or folder 00_00_00)
                    parts = name.split("_")
                    if len(parts) >= 2 and parts[1].isdigit():
                        idx = int(parts[1])
                        if 0 <= idx < len(RUSSIAN_CYRILLIC_LETTERS):
                            char_val = RUSSIAN_CYRILLIC_LETTERS[idx]
                    
                    if not char_val:
                        folder_parts = folder_name.split("_")
                        if len(folder_parts) >= 2 and folder_parts[1].isdigit():
                            idx = int(folder_parts[1])
                            if 0 <= idx < len(RUSSIAN_CYRILLIC_LETTERS):
                                char_val = RUSSIAN_CYRILLIC_LETTERS[idx]

                    if not char_val:
                        if folder_name in self.class_to_idx:
                            char_val = folder_name
                        elif "_" in name and name.split("_")[0] in self.class_to_idx:
                            char_val = name.split("_")[0]
                        elif len(name) >= 1 and name[0] in self.class_to_idx:
                            char_val = name[0]

                    if char_val and char_val in self.class_to_idx:
                        self.samples.append((img_path, self.class_to_idx[char_val]))

    def __len__(self) -> int:
        return len(self.samples)

    def __getitem__(self, idx: int) -> Tuple[torch.Tensor, int]:
        img_path, label = self.samples[idx]
        im = Image.open(img_path).convert("L")
        arr = np.array(im, dtype=np.float32)
        rng = arr.max() - arr.min()
        if rng > 10:
            arr = (arr - arr.min()) / rng
            if arr.mean() > 0.5:
                arr = 1.0 - arr
            arr = (arr > 0.35).astype(np.float32)
        else:
            arr = np.zeros_like(arr)
        image = Image.fromarray((arr * 255).astype(np.uint8))
        if self.transform:
            image = self.transform(image)
        return image, label


class CoMNISTDataset(Dataset):
    """
    Loads samples from gregvial/comnist.
    Images organized in class-specific subfolders.
    Handles CP437/UTF-8 zip encoding artifacts (e.g. ╨É -> А).
    Handwriting strokes are stored in the alpha channel.
    """
    def __init__(self, root_dir: str, class_to_idx: Dict[str, int], transform=None):
        self.root_dir = Path(root_dir)
        self.class_to_idx = class_to_idx
        self.transform = transform
        self.samples: List[Tuple[Path, int]] = []

        for folder in self.root_dir.glob("**/*"):
            if folder.is_dir():
                raw_name = folder.name.strip()
                char_name = raw_name.lower()

                # Try fixing CP437 encoded Cyrillic folder names
                try:
                    decoded = raw_name.encode('cp437').decode('utf-8').lower()
                    if decoded in self.class_to_idx:
                        char_name = decoded
                except Exception:
                    pass

                if char_name in self.class_to_idx:
                    label_idx = self.class_to_idx[char_name]
                    for ext in ("*.png", "*.jpg", "*.jpeg"):
                        for img_path in folder.glob(ext):
                            self.samples.append((img_path, label_idx))

    def __len__(self) -> int:
        return len(self.samples)

    def __getitem__(self, idx: int) -> Tuple[torch.Tensor, int]:
        img_path, label = self.samples[idx]
        im = Image.open(img_path)
        if im.mode == "RGBA":
            # CoMNIST stores black ink in alpha channel (0=bg, 255=stroke)
            image = im.split()[-1]
        else:
            image = im.convert("L")
            if np.array(image).mean() > 128:
                image = Image.fromarray(255 - np.array(image))
        if self.transform:
            image = self.transform(image)
        return image, label


def get_transforms():
    train_transform = transforms.Compose([
        transforms.Resize((28, 28)),
        transforms.RandomAffine(
            degrees=8,
            translate=(0.08, 0.08),
            scale=(0.90, 1.10),
            fill=0,
        ),
        transforms.ToTensor(),
        transforms.Lambda(lambda x: (x > 0.2).float()),
    ])

    val_transform = transforms.Compose([
        transforms.Resize((28, 28)),
        transforms.ToTensor(),
        transforms.Lambda(lambda x: (x > 0.2).float()),
    ])

    return train_transform, val_transform


def train(
    dataset_name: str = "all",
    epochs: int = 30,
    batch_size: int = 128,
    lr: float = 0.001,
    export_onnx: bool = True,
    copy_to_tauri: bool = True,
):
    device = torch.device("cuda" if torch.cuda.is_available() else "mps" if torch.backends.mps.is_available() else "cpu")
    print(f"[*] Training on device: {device}")

    classes = ALL_CYRILLIC_LETTERS
    class_to_idx = {char: i for i, char in enumerate(classes)}
    idx_to_class = {i: char for char, i in class_to_idx.items()}

    with open("cyrillic_classes.json", "w", encoding="utf-8") as f:
        json.dump(idx_to_class, f, ensure_ascii=False, indent=2)
    print(f"[*] Registered {len(classes)} Cyrillic alphabet classes in cyrillic_classes.json")

    train_transform, val_transform = get_transforms()
    datasets = []

    # 1. Russian Letters dataset
    if dataset_name in ("all", "russian"):
        try:
            print("[*] Fetching Russian Letters dataset via kagglehub...")
            ru_path = kagglehub.dataset_download("olgabelitskaya/handwritten-russian-letters")
            print(f"[*] Russian Letters downloaded to: {ru_path}")
            ru_ds = RussianLettersDataset(ru_path, class_to_idx, transform=train_transform)
            print(f"[*] Loaded {len(ru_ds)} samples from Russian Letters dataset")
            if len(ru_ds) > 0:
                datasets.append(ru_ds)
        except Exception as e:
            print(f"[!] Could not download/load Russian Letters dataset: {e}")

    # 2. CoMNIST dataset
    if dataset_name in ("all", "comnist"):
        try:
            print("[*] Fetching CoMNIST dataset via kagglehub...")
            comnist_path = kagglehub.dataset_download("gregvial/comnist")
            print(f"[*] CoMNIST downloaded to: {comnist_path}")
            comnist_ds = CoMNISTDataset(comnist_path, class_to_idx, transform=train_transform)
            print(f"[*] Loaded {len(comnist_ds)} samples from CoMNIST dataset")
            if len(comnist_ds) > 0:
                datasets.append(comnist_ds)
        except Exception as e:
            print(f"[!] Could not download/load CoMNIST dataset: {e}")

    if not datasets:
        print("[!] No external dataset samples found. Generating baseline model structure...")
        class SyntheticDataset(Dataset):
            def __init__(self, num_classes, transform=None):
                self.num_classes = num_classes
                self.transform = transform
            def __len__(self):
                return self.num_classes * 5
            def __getitem__(self, idx):
                label = idx % self.num_classes
                img = Image.new("L", (28, 28), color=0)
                if self.transform:
                    img = self.transform(img)
                else:
                    img = transforms.ToTensor()(img)
                return img, label
        full_dataset = SyntheticDataset(len(classes))
    else:
        full_dataset = ConcatDataset(datasets) if len(datasets) > 1 else datasets[0]

    print(f"[*] Total training samples: {len(full_dataset)}")
    train_size = int(0.85 * len(full_dataset))
    val_size = len(full_dataset) - train_size
    train_ds, val_ds = random_split(full_dataset, [train_size, val_size])

    train_loader = DataLoader(train_ds, batch_size=batch_size, shuffle=True, num_workers=2 if torch.cuda.is_available() else 0)
    val_loader = DataLoader(val_ds, batch_size=batch_size, shuffle=False, num_workers=2 if torch.cuda.is_available() else 0)

    model = AlphabetNet(num_classes=len(classes)).to(device)
    criterion = FocalLoss(gamma=2.0)
    optimizer = optim.Adam(model.parameters(), lr=lr)

    print(f"[*] Starting training for {epochs} epochs...")
    for epoch in range(epochs):
        model.train()
        running_loss = 0.0
        for images, labels in train_loader:
            images, labels = images.to(device), labels.to(device)
            optimizer.zero_grad()
            outputs = model(images)
            loss = criterion(outputs, labels)
            loss.backward()
            optimizer.step()
            running_loss += loss.item()

        model.eval()
        correct, total = 0, 0
        with torch.no_grad():
            for images, labels in val_loader:
                images, labels = images.to(device), labels.to(device)
                outputs = model(images)
                _, predicted = torch.max(outputs.data, 1)
                total += labels.size(0)
                correct += (predicted == labels).sum().item()

        val_acc = (100.0 * correct / total) if total > 0 else 0.0
        avg_loss = (running_loss / len(train_loader)) if len(train_loader) > 0 else 0.0
        print(f"Epoch [{epoch+1:02d}/{epochs:02d}] - Loss: {avg_loss:.4f} - Val Acc: {val_acc:.2f}%")

    if export_onnx:
        onnx_filename = "cyrillic_model.onnx"
        print(f"[*] Exporting final ONNX model to {onnx_filename}...")
        model.eval()
        dummy_input = torch.randn(1, 1, 28, 28, device=device)
        torch.onnx.export(
            model,
            dummy_input,
            onnx_filename,
            export_params=True,
            opset_version=14,
            do_constant_folding=True,
            input_names=["input"],
            output_names=["output"],
            dynamic_axes={"input": {0: "batch_size"}, "output": {0: "batch_size"}},
        )
        print(f"[SUCCESS] Exported {onnx_filename}")

        if copy_to_tauri:
            tauri_models_dir = Path(__file__).resolve().parent.parent / "parlezvous" / "src-tauri" / "models"
            if tauri_models_dir.exists():
                dest_onnx = tauri_models_dir / onnx_filename
                dest_json = tauri_models_dir / "cyrillic_classes.json"
                shutil.copyfile(onnx_filename, dest_onnx)
                shutil.copyfile("cyrillic_classes.json", dest_json)
                data_file = onnx_filename + ".data"
                if os.path.exists(data_file):
                    shutil.copyfile(data_file, tauri_models_dir / data_file)
                print(f"[COPIED] Copied model, data & classes to: {tauri_models_dir}")


def main():
    parser = argparse.ArgumentParser(description="Train Cyrillic handwriting recognition model")
    parser.add_argument("--dataset", choices=["all", "russian", "comnist"], default="all")
    parser.add_argument("--epochs", type=int, default=10)
    parser.add_argument("--batch-size", type=int, default=256)
    parser.add_argument("--lr", type=float, default=0.001)
    parser.add_argument("--no-copy", action="store_true", help="Do not copy model to parlezvous/src-tauri/models")
    args = parser.parse_args()

    train(
        dataset_name=args.dataset,
        epochs=args.epochs,
        batch_size=args.batch_size,
        lr=args.lr,
        export_onnx=True,
        copy_to_tauri=not args.no_copy,
    )


if __name__ == "__main__":
    main()
