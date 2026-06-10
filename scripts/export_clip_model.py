"""
Export OpenCLIP ViT-B-32 image encoder to ONNX and pre-compute text embeddings.
Cross-platform: works on Windows, macOS, Linux.

Usage:
  python scripts/export_clip_model.py

On Windows, if you hit OPENSSL_Uplink errors, run via the wrapper:
  scripts/run_export_model.sh        (Git Bash / WSL)
  scripts\run_export_model.ps1       (PowerShell)
"""
import os
import sys
import numpy as np
import torch
import open_clip

OUTPUT_DIR = os.path.join(os.path.dirname(os.path.abspath(__file__)), '..', 'models')
os.makedirs(OUTPUT_DIR, exist_ok=True)

MODEL_NAME = 'ViT-B-32'
PRETRAINED = 'openai'

PROMPTS = [
    "a meme or emoji sticker image",
    "a blurry candid photo with distorted facial expression",
    "a severely backlit portrait with dark face",
    "a photo with lens distortion warping faces at edges",
    "a normal good quality photograph",
]

LABELS = [
    "meme_emoji",
    "bad_expression_blur",
    "backlit",
    "lens_distortion",
    "normal",
]

print(f"Loading OpenCLIP {MODEL_NAME} ({PRETRAINED})...")
model, _, preprocess = open_clip.create_model_and_transforms(MODEL_NAME, pretrained=PRETRAINED)
tokenizer = open_clip.get_tokenizer(MODEL_NAME)
model.eval()

# Export image encoder to ONNX
print("Exporting image encoder to ONNX...")
dummy_image = torch.randn(1, 3, 224, 224)

class ImageEncoderWrapper(torch.nn.Module):
    def __init__(self, clip_model):
        super().__init__()
        self.visual = clip_model.visual

    def forward(self, x):
        features = self.visual(x)
        return features / features.norm(dim=-1, keepdim=True)

encoder = ImageEncoderWrapper(model)
encoder.eval()

onnx_path = os.path.join(OUTPUT_DIR, 'clip_image_encoder.onnx')
torch.onnx.export(
    encoder,
    dummy_image,
    onnx_path,
    input_names=['image'],
    output_names=['embedding'],
    dynamic_axes={'image': {0: 'batch'}, 'embedding': {0: 'batch'}},
    opset_version=14,
)
print(f"  Saved: {onnx_path} ({os.path.getsize(onnx_path) / 1024 / 1024:.1f} MB)")

# Pre-compute text embeddings
print("Computing text embeddings...")
text_tokens = tokenizer(PROMPTS)
with torch.no_grad():
    text_features = model.encode_text(text_tokens)
    text_features = text_features / text_features.norm(dim=-1, keepdim=True)

embeddings = text_features.numpy().astype(np.float32)
bin_path = os.path.join(OUTPUT_DIR, 'text_embeddings.bin')
embeddings.tofile(bin_path)
print(f"  Saved: {bin_path} ({embeddings.shape[0]} prompts x {embeddings.shape[1]} dims)")

labels_path = os.path.join(OUTPUT_DIR, 'labels.txt')
with open(labels_path, 'w') as f:
    for label in LABELS:
        f.write(label + '\n')
print(f"  Saved: {labels_path}")

print("\nDone! Model files ready in:", os.path.abspath(OUTPUT_DIR))
