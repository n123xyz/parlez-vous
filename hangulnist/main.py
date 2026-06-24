import os
import json
import torch
import torch.nn as nn
import torch.optim as optim
from torch.utils.data import Dataset, DataLoader, random_split
from torchvision import transforms
from PIL import Image
import kagglehub
import torch.nn.functional as F

# ---------------------------------------------------------
# 0. Focal Loss for Hard Examples
# ---------------------------------------------------------
class FocalLoss(nn.Module):
    def __init__(self, gamma=2.0):
        super().__init__()
        self.gamma = gamma

    def forward(self, inputs, targets):
        ce_loss = F.cross_entropy(inputs, targets, reduction='none')
        pt = torch.exp(-ce_loss)
        focal_loss = ((1 - pt) ** self.gamma) * ce_loss
        return focal_loss.mean()

# ---------------------------------------------------------
# 1. Custom Dataset for Flat Folder (Prefix Labels)
# ---------------------------------------------------------
class HangulFlatDataset(Dataset):
    def __init__(self, folder_path, transform=None):
        self.folder_path = folder_path
        self.transform = transform

        all_files = [f for f in os.listdir(folder_path) if f.endswith(('.png', '.jpg', '.jpeg'))]
        self.valid_files = [f for f in all_files if '_' in f]

        self.classes = sorted(list(set([f.split('_')[0] for f in self.valid_files])))
        self.class_to_idx = {cls_name: i for i, cls_name in enumerate(self.classes)}

    def __len__(self):
        return len(self.valid_files)

    def __getitem__(self, idx):
        file_name = self.valid_files[idx]
        img_path = os.path.join(self.folder_path, file_name)

        image = Image.open(img_path).convert("L")

        label_str = file_name.split('_')[0]
        label_idx = self.class_to_idx[label_str]

        if self.transform:
            image = self.transform(image)

        return image, label_idx

# ---------------------------------------------------------
# 2. Upgraded CNN Architecture (3-Layer + BatchNorm)
# ---------------------------------------------------------
class HangulNet(nn.Module):
    def __init__(self, num_classes):
        super().__init__()
        # Input: [Batch, 1, 28, 28]
        self.features = nn.Sequential(
            # Block 1: Look at the image at FULL resolution for longer
            nn.Conv2d(1, 32, kernel_size=3, padding=1),
            nn.BatchNorm2d(32),
            nn.ReLU(),
            nn.Conv2d(32, 32, kernel_size=3, padding=1), # <-- NEW: Second Conv before pooling
            nn.BatchNorm2d(32),
            nn.ReLU(),
            nn.MaxPool2d(2), # Output: [32, 14, 14]

            # Block 2
            nn.Conv2d(32, 64, kernel_size=3, padding=1),
            nn.BatchNorm2d(64),
            nn.ReLU(),
            nn.MaxPool2d(2), # Output: [64, 7, 7]

            # Block 3
            nn.Conv2d(64, 128, kernel_size=3, padding=1),
            nn.BatchNorm2d(128),
            nn.ReLU(),
            # No pooling here to preserve the 7x7 grid
        )

        self.classifier = nn.Sequential(
            nn.Flatten(),
            nn.Linear(128 * 7 * 7, 256),
            nn.BatchNorm1d(256), # Added BatchNorm for stability
            nn.ReLU(),
            nn.Dropout(0.3),     # Lowered dropout slightly for 28x28
            nn.Linear(256, num_classes)
        )

    def forward(self, x):
        x = self.features(x)
        x = self.classifier(x)
        return x

# ---------------------------------------------------------
# 3. Main Training & Export Pipeline
# ---------------------------------------------------------
def main():
    device = torch.device("cuda" if torch.cuda.is_available() else "mps" if torch.backends.mps.is_available() else "cpu")
    print(f"[*] Training on device: {device}")

    print("[*] Downloading dataset via Kagglehub...")
    base_path = kagglehub.dataset_download("wayperwayp/hangulkorean-characters")

    dataset_path = os.path.join(base_path, "hangul_characters_v1")
    if not os.path.exists(dataset_path):
        dataset_path = base_path

    print(f"[*] Dataset ready at: {dataset_path}")

    # Step 2: Gentler Transforms
    # Removed GaussianBlur and Shear. Reduced Rotation and Translation.
    # The transform pipeline
    train_transform = transforms.Compose([
        transforms.Resize((28, 28)),
        transforms.RandomAffine(
            degrees=5,
            translate=(0.08, 0.08),
            scale=(0.90, 1.10),
        ),
        transforms.ToTensor(),
        transforms.Lambda(lambda x: 1.0 - x), # Invert (background black, strokes white)
        transforms.Lambda(lambda x: (x > 0.3).float()) # <-- NEW: Sharpen! If pixel is slightly white, make it pure white. Otherwise, pure black.
    ])

    val_transform = transforms.Compose([
        transforms.Resize((28, 28)),
        transforms.ToTensor(),
        transforms.Lambda(lambda x: 1.0 - x),
        transforms.Lambda(lambda x: (x > 0.3).float()) # <-- NEW: Sharpen!
    ])

    full_dataset = HangulFlatDataset(folder_path=dataset_path, transform=train_transform)
    num_classes = len(full_dataset.classes)

    class_map = {idx: char for char, idx in full_dataset.class_to_idx.items()}
    with open("hangul_classes.json", "w", encoding="utf-8") as f:
        json.dump(class_map, f, ensure_ascii=False, indent=2)
    print(f"[*] Found {num_classes} unique characters.")

    train_size = int(0.85 * len(full_dataset))
    val_size = len(full_dataset) - train_size
    train_dataset, val_dataset = random_split(full_dataset, [train_size, val_size])

    val_dataset.dataset = HangulFlatDataset(folder_path=dataset_path, transform=val_transform)

    train_loader = DataLoader(train_dataset, batch_size=128, shuffle=True, num_workers=2)
    val_loader = DataLoader(val_dataset, batch_size=128, shuffle=False, num_workers=2)

    model = HangulNet(num_classes).to(device)
    criterion = FocalLoss(gamma=2.0)
    optimizer = optim.Adam(model.parameters(), lr=0.001)

    epochs = 30
    print("[*] Starting training...")
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

        val_acc = 100 * correct / total
        print(f"Epoch [{epoch+1}/{epochs}] - Loss: {running_loss/len(train_loader):.4f} - Val Acc: {val_acc:.2f}%")

    print("[*] Exporting model to ONNX...")
    model.eval()

    dummy_input = torch.randn(1, 1, 28, 28, device=device)
    onnx_path = "character_model.onnx"

    torch.onnx.export(
        model,
        dummy_input,
        onnx_path,
        export_params=True,
        opset_version=14,
        do_constant_folding=True,
        input_names=['input'],
        output_names=['output'],
        dynamic_axes={'input': {0: 'batch_size'}, 'output': {0: 'batch_size'}}
    )

    print(f"[SUCCESS] Model exported to {onnx_path}")

if __name__ == "__main__":
    main()
