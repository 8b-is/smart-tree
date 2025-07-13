FROM python:3.11-slim

LABEL maintainer="8b-is <st@8b.is>"
LABEL description="Smart Tree Feedback Worker - Mean ass efficient feedback processor"

# Install system deps
RUN apt-get update && apt-get install -y --no-install-recommends \
    gcc \
    libc6-dev \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN groupadd -r feedback && useradd -r -g feedback feedback

WORKDIR /app

# Install Python dependencies
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

# Copy application
COPY worker.py .

# Switch to non-root user
USER feedback

# Expose metrics port
EXPOSE 9090

# Run worker
CMD ["python", "-u", "worker.py"]