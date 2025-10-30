# GestureSpace Project Presentation

This directory contains a [Slidev](https://sli.dev/) presentation for the GestureSpace project.

## 📖 Overview

**Duration**: 4.5 minutes (270 seconds)

**Structure**:
1. Market Current Situation (30 seconds)
2. GestureSpace Core Techniques (2 minutes)
3. Navigation System - Navign Integration (1 minute)
4. GestureSpace-Powered Robot (1.5 minutes)

## 🚀 Getting Started

### Prerequisites

- Node.js 18+
- pnpm (or npm)

### Installation

```bash
cd presentation
pnpm install
```

### Development

Start the presentation in development mode with hot reload:

```bash
pnpm dev
```

This will open the presentation at `http://localhost:3030`

### Build

Build the presentation for production:

```bash
pnpm build
```

The static files will be generated in the `dist` directory.

### Export

Export slides as PDF:

```bash
pnpm export
```

Export as PNG images:

```bash
pnpm export --format png
```

## 🎨 Presentation Features

- **Interactive slides** with smooth transitions
- **Code highlighting** with Shiki
- **Mermaid diagrams** for architecture visualization
- **Click animations** (v-clicks) for progressive disclosure
- **Two-column layouts** for comparisons
- **Mobile-responsive** design

## 📝 Content Sections

### 1. Market Current Situation
- China's accessibility crisis (17.31M visually impaired, 400 guide dogs)
- Indoor navigation challenges
- Current solution limitations

### 2. GestureSpace Core Techniques
- Hand Gesture Recognition (MediaPipe + CNN)
- Finger Pointing Direction Detection
- Object Detection (YOLOv8)
- 3D Localization & Camera Pose Estimation
- Voice Wake Word Detection (Porcupine)
- Speech Recognition & Response

### 3. Navigation System (Navign)
- BLE beacon infrastructure
- RSSI-based positioning
- P-256 ECDSA security
- Advanced pathfinding (Dijkstra)
- Cross-platform mobile app

### 4. GestureSpace-Powered Robot
- Dual-layer architecture (ROS2 + STM32)
- Vision-based control
- Autonomous navigation
- Guide mode for visually impaired
- Delivery mode

## 🎮 Keyboard Shortcuts

- `Space` / `→`: Next slide
- `←`: Previous slide
- `d`: Toggle dark mode
- `o`: Toggle slides overview
- `f`: Toggle fullscreen
- `g`: Go to specific slide (type number)

## 🛠️ Customization

Edit `slides.md` to customize the presentation content. The file uses Markdown with Slidev extensions.

### Themes

To change the theme, modify the frontmatter in `slides.md`:

```yaml
---
theme: default  # or: seriph, apple-basic, etc.
---
```

Available themes: https://sli.dev/themes/gallery.html

## 📚 Resources

- [Slidev Documentation](https://sli.dev/)
- [GestureSpace Project](https://github.com/indoor-mall-nav/navign)
- [Presentation Outline](../PRESENTATION_OUTLINE.md)

## 📄 License

MIT
