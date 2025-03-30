# Step 1: Use an official Python runtime as the base image
FROM python:3.12-slim

# Set working directory in the container
WORKDIR /app

# Install the required dependencies
COPY requirements.txt ./
RUN pip install --no-cache-dir -r requirements.txt

# Copy the rest of the controller code
COPY . .

# Expose port 8081 (the controller port)
EXPOSE 8081

# Run the FastAPI app with Uvicorn (controller listens on port 8081)
CMD ["uvicorn", "controller:app", "--host", "0.0.0.0", "--port", "8081"]
