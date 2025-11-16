# Robot Intelligence Service

AI-powered natural language interaction for accessibility - converts visual scene data into human-friendly descriptions for visually impaired users.

## Overview

**Hybrid local/remote LLM architecture:**
- **Local-first:** Qwen3-0.6B for fast, offline inference
- **Cloud fallback:** GPT-4o/DeepSeek for complex queries
- **Geo-aware:** Auto-selects API based on region

## Architecture

```
User Query + Scene Data
         │
         ▼
   Local LLM (Qwen3-0.6B)
         │
    Success? ──► Return
         │
    <remote> ──► GPT-4o/DeepSeek ──► Return
```

## Files

- **`local.py`** - Qwen3-0.6B inference, outputs `<remote>` when uncertain
- **`remote.py`** - GPT-4o/DeepSeek fallback with geo-aware routing
- **`shared.py`** - System prompts and configuration

## Use Case

**Accessibility for visually impaired users:**

1. Vision detects objects: `chair (2.5, 1.0, 0.0), table (3.0, 0.5, 0.0)`
2. Intelligence converts: "There's a chair nearby to your right, a table slightly further"
3. Audio speaks description via TTS

**Key feature:** Converts coordinates to spatial language ("near", "far", "left", "right")

## Setup

```bash
cd robot/intelligence

# Install dependencies
uv sync

# Configure API keys
cp config.example.py config.py
# Edit config.py: Add OPENAI_KEY and DEEPSEEK_KEY
```

## Dependencies

```toml
transformers>=4.57.1    # Hugging Face models
openai>=2.8.0          # OpenAI API
eclipse-zenoh>=1.6.2   # Message bus (future)
protobuf>=6.33.1       # Protocol buffers (future)
```

## Usage

```python
from local import generate_response

scene = "chair at (2, 1, 0), table at (3, 0, 0.8)"
response = generate_response(scene, "What's around me?")
# "There's a chair close by to your right, a table ahead..."
```

### API Selection Logic

```python
# Checks IP geolocation via ip.sb
# OpenAI: Most regions
# DeepSeek: China, Hong Kong, Russia, Iran, etc.
```

## Performance

| Mode | Latency | Cost | Offline |
|------|---------|------|---------|
| Local (Qwen3) | 100-500ms | Free | ✅ Yes |
| Remote (GPT-4o) | 1-3s | ~$0.01-0.03 | ❌ No |
| Remote (DeepSeek) | 2-5s | ~$0.001-0.002 | ❌ No |

**Memory:** ~2-4 GB RAM for local model

## Integration (Future)

**Zenoh topics:**
- Subscribe: `robot/vision/objects`, `robot/audio/user_query`
- Publish: `robot/intelligence/descriptions`

```python
# service.py (to be implemented)
import zenoh
from local import generate_response

session = zenoh.open()
session.declare_subscriber("robot/vision/objects", callback)
publisher = session.declare_publisher("robot/intelligence/descriptions")
```

## Testing

```bash
# Test local
uv run python -c "from local import generate_response; \
  print(generate_response('chair at (2,1,0)', 'What is here?'))"

# Test remote
uv run python -c "from remote import run_remote_response; \
  print(run_remote_response('chair at (2,1,0)', 'Describe'))"
```

## Limitations

- **Local:** Limited reasoning, outputs `<remote>` when uncertain
- **Remote:** Requires internet, API costs, regional restrictions

## Troubleshooting

**Out of memory:** Use CPU or quantized model
**API errors:** Check `config.py` keys and regional availability
**Model download:** Pre-download via `transformers.AutoModelForCausalLM.from_pretrained('Qwen/Qwen3-0.6B')`

## See Also

- [Robot Upper Layer](../README.md)
- [Vision Service](../vision/README.md)
- [Audio Service](../audio/README.md)
- [Qwen3 Model](https://huggingface.co/Qwen/Qwen3-0.6B)

---

**Status:** ⚠️ Core implemented, Zenoh integration pending | **License:** MIT | **Updated:** 2025-11-16
