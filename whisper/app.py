import whisper
import uvicorn
from fastapi import FastAPI, UploadFile, File, Form, HTTPException
from fastapi.middleware.cors import CORSMiddleware
import tempfile
import os
import torch
import asyncio
from typing import Optional

# Check for GPU
device = "cuda" if torch.cuda.is_available() else "cpu"

# --- Model Loading ---
# Note: Whisper model sizes include: "tiny", "base", "small", "medium", "large".
# Larger models require more memory and compute.
model_name = "tiny"
print(f"Loading Whisper model '{model_name}' on device '{device}'...")
model = whisper.load_model(model_name, device=device)
print("Whisper model loaded.")

# Optional: Add a lock to prevent GPU Out-Of-Memory errors from concurrent requests
inference_lock = asyncio.Lock()
# --- End Model Loading ---

app = FastAPI()

app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

@app.get("/")
def read_root():
    return {"message": "Whisper ASR API is running. Use the /v1/audio/transcriptions endpoint."}

@app.get("/v1/models")
def list_models():
    return {
        "object": "list",
        "data": [
            {
                "id": "whisper-1",
                "object": "model",
                "created": 0,
                "owned_by": "self-hosted",
                "permission": [],
                "root": "whisper-1",
                "parent": None,
            }
        ]
    }

# Helper function to wrap the blocking Whisper call
def run_whisper_transcription(file_path: str, options: dict) -> dict:
    import gc
    try:
        with torch.no_grad():
            # whisper.transcribe automatically handles language detection if 'language' is None
            return model.transcribe(file_path, **options)
    finally:
        if torch.cuda.is_available():
            torch.cuda.empty_cache()
        gc.collect()

@app.post("/v1/audio/transcriptions")
async def transcribe_audio(
    file: UploadFile = File(...),
    requested_model: str = Form("whisper-1"),
    language: Optional[str] = Form(None),
    prompt: Optional[str] = Form(None),
    response_format: Optional[str] = Form("json"),
    temperature: Optional[float] = Form(0.0)
):
    print(f"Received transcription request for {file.filename}, format={response_format}")
    
    # Save uploaded file to disk
    with tempfile.NamedTemporaryFile(delete=False, suffix=os.path.splitext(file.filename)[1]) as temp_file:
        content = await file.read()
        temp_file.write(content)
        temp_file_path = temp_file.name

    try:
        options = {
            "fp16": device == "cuda",
            "language": language, # If None, Whisper auto-detects
            "prompt": prompt,
            "temperature": temperature,
        }        
        # Remove None values
        options = {k: v for k, v in options.items() if v is not None}

        # Safely run the heavy ML model in a background thread, protected by a lock
        async with inference_lock:
            result = await asyncio.to_thread(run_whisper_transcription, temp_file_path, options)
            
        print("Transcription successful.")
        transcribed_text = result["text"].strip()

        if response_format == "json":
            return {"text": transcribed_text}
        elif response_format == "text":
            return transcribed_text
        elif response_format == "srt":
            return srt_content(result)
        elif response_format == "vtt":
            return vtt_content(result)
        else:
            raise HTTPException(status_code=400, detail="Unsupported response_format")

    except Exception as e:
        print(f"Error during transcription: {e}")
        raise HTTPException(status_code=500, detail=str(e))
    finally:
        # Ensure cleanup happens no matter what
        if os.path.exists(temp_file_path):
            os.remove(temp_file_path)

# --- Format Helpers ---
def format_timestamp(seconds: float, always_include_hours: bool = True, decimal_marker: str = ","):
    assert seconds >= 0, "non-negative timestamp expected"
    milliseconds = round(seconds * 1000.0)
    hours = milliseconds // 3_600_000
    milliseconds -= hours * 3_600_000
    minutes = milliseconds // 60_000
    milliseconds -= minutes * 60_000
    seconds = milliseconds // 1_000
    milliseconds -= seconds * 1_000
    hours_marker = f"{hours:02d}:" if always_include_hours or hours > 0 else ""
    return f"{hours_marker}{minutes:02d}:{seconds:02d}{decimal_marker}{milliseconds:03d}"

def srt_content(result: dict) -> str:
    srt_lines = []
    for i, segment in enumerate(result["segments"], start=1):
        start_time = format_timestamp(segment["start"], decimal_marker=",")
        end_time = format_timestamp(segment["end"], decimal_marker=",")
        srt_lines.append(f"{i}")
        srt_lines.append(f"{start_time} --> {end_time}")
        srt_lines.append(f"{segment['text'].strip()}\n")
    return "\n".join(srt_lines)

def vtt_content(result: dict) -> str:
    vtt_lines = ["WEBVTT\n"]
    for segment in result["segments"]:
        start_time = format_timestamp(segment["start"], always_include_hours=False, decimal_marker=".")
        end_time = format_timestamp(segment["end"], always_include_hours=False, decimal_marker=".")
        vtt_lines.append(f"{start_time} --> {end_time}")
        vtt_lines.append(f"{segment['text'].strip()}\n")
    return "\n".join(vtt_lines)

if __name__ == "__main__":
    uvicorn.run(app, host="0.0.0.0", port=8000, timeout_keep_alive=5)
