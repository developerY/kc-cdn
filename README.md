# 📦 kc-cdn: KoColor Static Asset CDN

## 🛑 Overview
This repository serves as the **Content Delivery Network (CDN)** for the KoColor Android application. Hosted freely via GitHub Pages, it stores and delivers the highly optimized product images (cosmetics, skincare, and clothing) required by the KoColor offline-first taxonomy. 

By separating the static storage (`kc-cdn`) from the active compute environment (Rust Axum server on Hugging Face), we maintain a zero-cost, high-performance, and scalable backend architecture.

---

## 📐 Asset Specifications
To guarantee blazing-fast downloads and prevent local Android Out-Of-Memory (OOM) errors in the Jetpack Compose UI, all images committed to this repository **must** adhere strictly to the following rules:

* **Dimensions:** 512 x 512 pixels (1:1 Square Ratio)
* **Format:** WebP (`.webp`)
* **Target Size:** 30 KB – 50 KB
* **Naming Convention:** Must match the exact filename referenced in the Hugging Face Rust payload (e.g., `image_07157d.webp` or `cosmetic_lip_001.webp`).

---

## ⚙️ Batch Optimization Script (Python)
Because AI-generated assets are typically large, high-resolution formats (e.g., 1024x1024 JPEGs or PNGs), they must be resized and compressed before being committed here to preserve the 1 GB repository limit.

You can use the following Python script (utilizing the `Pillow` library) locally to instantly convert a staging directory of raw images into production-ready `.webp` files.

### 1. Install Dependencies
```bash
pip install Pillow
