import os
import io
import sys
import argparse
import uvicorn
import torch
import numpy as np
import soundfile as sf
import logging
import gc
import hashlib
from datetime import datetime
from contextlib import asynccontextmanager
from typing import Optional, List

from fastapi import FastAPI, Response, HTTPException
from fastapi.responses import JSONResponse
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel

sys.path.append(os.path.abspath(os.path.join(os.path.dirname(__file__), '..')))
from qwen_tts import Qwen3TTSModel

# --- Logging Setup ---
logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s [%(levelname)s] %(message)s",
    datefmt="%Y-%m-%d %H:%M:%S"
)
logger = logging.getLogger("Qwen3-TTS-Server")

# --- Pydantic Models ---
class OpenAISpeechRequest(BaseModel):
    model: Optional[str] = "tts-1"
    input: str
    voice: str  # Preset speaker ID (e.g., 'alloy', 'nova')
    response_format: Optional[str] = "wav"
    speed: Optional[float] = 1.0

# --- Global State ---
model_wrapper: Optional[Qwen3TTSModel] = None

# --- Lifespan Event ---
@asynccontextmanager
async def lifespan(app: FastAPI):
    global model_wrapper
    logger.info("Loading Qwen3-TTS Base Model...")

    ckpt = os.environ.get("QWEN_TTS_CHECKPOINT")
    if not ckpt:
        logger.error("Error: QWEN_TTS_CHECKPOINT environment variable not set.")
        exit(1)

    # Ensure output directory exists on startup
    output_dir = os.environ.get("QWEN_TTS_OUTPUT_DIR", "outputs")
    os.makedirs(output_dir, exist_ok=True)
    logger.info(f"Audio files will be saved to: {os.path.abspath(output_dir)}")

    device = os.environ.get("QWEN_TTS_DEVICE", "cuda:0" if torch.cuda.is_available() else "cpu")
    dtype_str = os.environ.get("QWEN_TTS_DTYPE", "bfloat16")

    if dtype_str == "bfloat16":
        dtype = torch.bfloat16
    elif dtype_str == "float16":
        dtype = torch.float16
    else:
        dtype = torch.float32

    attn_impl = os.environ.get("QWEN_TTS_ATTN", "flash_attention_2" if torch.cuda.is_available() and "cuda" in device else None)
    if attn_impl == "none":
        attn_impl = None

    try:
        model_wrapper = Qwen3TTSModel.from_pretrained(
            ckpt,
            device_map=device,
            dtype=dtype,
            attn_implementation=attn_impl,
        )
        logger.info(f"Model loaded successfully on {device} with {dtype_str}.")
    except Exception as e:
        logger.error(f"Failed to load model: {e}")
        exit(1)

    yield
    logger.info("Shutting down...")

app = FastAPI(lifespan=lifespan)

app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# --- Helpers ---
def _wav_to_bytes(wav: np.ndarray, sr: int, format: str = "WAV") -> bytes:
    with io.BytesIO() as wav_buffer:
        sf.write(wav_buffer, wav, sr, format=format)
        return wav_buffer.getvalue()

def split_text(text: str, max_chars: int = 300) -> List[str]:
    """
    Simple text splitter to handle long inputs.
    Splits by basic punctuation to avoid cutting words.
    """
    if len(text) <= max_chars:
        return [text]

    chunks = []
    current_chunk = ""

    sentences = text.replace("。", ".").replace("！", "!").replace("？", "?").replace("\n", " ").split(". ")

    for sentence in sentences:
        if len(current_chunk) + len(sentence) < max_chars:
            current_chunk += sentence + ". "
        else:
            if current_chunk:
                chunks.append(current_chunk.strip())
            current_chunk = sentence + ". "

    if current_chunk:
        chunks.append(current_chunk.strip())

    return chunks

# --- Endpoints ---
@app.get("/v1/health")
async def health_check():
    if model_wrapper is None:
        return JSONResponse(status_code=503, content={"status": "unhealthy", "message": "Model not loaded"})
    return JSONResponse(status_code=200, content={"status": "healthy"})

@app.post("/v1/audio/speech")
async def openai_speech(req: OpenAISpeechRequest):
    if model_wrapper is None:
        raise HTTPException(status_code=503, detail="Model not initialized")

    try:
        logger.info(f"Request: input_len={len(req.input)}, voice={req.voice}")

        texts = split_text(req.input)
        if len(texts) > 1:
            logger.info(f"Split input into {len(texts)} chunks.")

        language = "Auto"

        # Base generation
        with torch.no_grad():
            wavs, sr = model_wrapper.generate_custom_voice(
                text=texts,
                speaker=[req.voice] * len(texts),
                language=[language] * len(texts)
            )

        if not wavs:
             raise HTTPException(status_code=500, detail="No audio generated.")

        final_wav = np.concatenate(wavs)

        # --- SAVE TO DISK LOGIC ---
        output_dir = os.environ.get("QWEN_TTS_OUTPUT_DIR", "outputs")
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        text_hash = hashlib.md5(req.input.encode('utf-8')).hexdigest()[:6]

        filename = f"speech_{timestamp}_{text_hash}.wav"
        file_path = os.path.join(output_dir, filename)

        # Save the physical file using soundfile
        sf.write(file_path, final_wav, sr)
        logger.info(f"Saved audio snippet to disk: {file_path}")
        # ---------------------------

        # Return the bytes to the HTTP response
        audio_bytes = _wav_to_bytes(final_wav, sr, format="WAV")
        return Response(content=audio_bytes, media_type="audio/wav")

    except Exception as e:
        logger.error(f"Generation error: {e}", exc_info=True)
        raise HTTPException(status_code=500, detail=str(e))

    finally:
        if torch.cuda.is_available():
            torch.cuda.empty_cache()
        gc.collect()

@app.get("/v1/models")
async def list_models():
    if model_wrapper is None:
         return JSONResponse(status_code=503, content={"error": "Model not loaded"})

    ckpt_name = os.environ.get("QWEN_TTS_CHECKPOINT", "unknown")

    return {
        "object": "list",
        "data": [
            {
                "id": "tts-1",
                "object": "model",
                "created": int(datetime.now().timestamp()),
                "owned_by": "qwen",
                "permission": [],
                "root": "tts-1",
                "parent": None,
            },
            {
                "id": ckpt_name,
                "object": "model",
                "created": int(datetime.now().timestamp()),
                "owned_by": "qwen",
                "permission": [],
                "root": ckpt_name,
                "parent": None,
            }
        ]
    }

def main():
    parser = argparse.ArgumentParser(description="Qwen3-TTS OpenAI Compatible Server (Base Only)")

    parser.add_argument("--checkpoint", default="Qwen/Qwen3-TTS-0.6B", help="Path or HF ID of the model checkpoint")
    parser.add_argument("--host", default="0.0.0.0", help="Host to bind to")
    parser.add_argument("--port", type=int, default=8081, help="Port to bind to")
    parser.add_argument("--device", default="cuda:0", help="Device (e.g., cuda:0, cpu)")
    parser.add_argument("--dtype", default="bfloat16", choices=["bfloat16", "float16", "float32"], help="Data type to load the model in")
    parser.add_argument("--no-flash-attn", action="store_true", help="Disable FlashAttention")

    # NEW ARGUMENT FOR SAVING TO DISK
    parser.add_argument("--output-dir", default="outputs", help="Directory to save generated audio snippets")

    args = parser.parse_args()

    os.environ["QWEN_TTS_CHECKPOINT"] = args.checkpoint
    os.environ["QWEN_TTS_DEVICE"] = args.device
    os.environ["QWEN_TTS_DTYPE"] = args.dtype
    os.environ["QWEN_TTS_OUTPUT_DIR"] = args.output_dir

    if args.no_flash_attn:
        os.environ["QWEN_TTS_ATTN"] = "none"

    uvicorn.run(app, host=args.host, port=args.port, timeout_keep_alive=5)

if __name__ == "__main__":
    main()
