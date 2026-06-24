# 1. Use an official Python runtime as a parent image
FROM python:3.10-slim

# 2. Set the working directory
WORKDIR /app

# 3. Install system dependencies
# ffmpeg is required by whisper for audio processing
RUN apt-get update && apt-get install -y --no-install-recommends \
    ffmpeg \
    && rm -rf /var/lib/apt/lists/*

# 4. Copy the entire project source code into the image
COPY . .

# 5. Install Python dependencies
# First, install dependencies from your requirements.txt
RUN pip install --no-cache-dir -r requirements.txt

# 6. Install the new server dependencies
RUN pip install --no-cache-dir "fastapi[all]" uvicorn python-multipart

RUN pip install --no-cache-dir spacy    
RUN python -m spacy download en_core_web_sm

# 7. Install the local whisper package
RUN pip install .

# 8. Expose the port the app runs on
EXPOSE 8000

# 9. Define the command to run the application
CMD ["uvicorn", "app:app", "--host", "0.0.0.0", "--port", "8000"]
