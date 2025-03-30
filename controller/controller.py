from fastapi import FastAPI, HTTPException
import requests
from pydantic import BaseModel
from typing import Optional
from fastapi.middleware.cors import CORSMiddleware

# Define the FastAPI app
app = FastAPI()

#Cors config
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],  # Allow all origins
    allow_credentials=True,
    allow_methods=["*"],  # Allow all HTTP methods (GET, POST, etc.)
    allow_headers=["*"],  # Allow all headers
)

# Request body model to handle test parameters
class TestParams(BaseModel):
    intensity: Optional[int] = 4  # Default CPU threads
    duration: Optional[int] = 10  # Default duration
    load: Optional[float] = 100.0  # Default load percentage
    size: Optional[int] = 256  # Default size for memory and disk tests
    fork: Optional[bool] = False  # Fork stress flag

# Function to send CPU stress request to the backend engine
def send_cpu_stress_request(test_params: TestParams):
    url = "http://engine-service:8080/cpu-stress"
    request_data = {
        "intensity": test_params.intensity,
        "duration": test_params.duration,
        "load": test_params.load,
        "fork": test_params.fork,
    }
    response = requests.post(url, json=request_data)
    if response.status_code != 200:
        raise HTTPException(status_code=response.status_code, detail="Failed to start CPU stress test")
    return response.text

# Function to send memory stress request to the backend engine
def send_memory_stress_request(test_params: TestParams):
    url = "http://engine-service:8080/mem-stress"
    request_data = {
        "size": test_params.size,
        "duration": test_params.duration,
    }
    response = requests.post(url, json=request_data)
    if response.status_code != 200:
        raise HTTPException(status_code=response.status_code, detail="Failed to start memory stress test")
    return response.text

# Function to send disk stress request to the backend engine
def send_disk_stress_request(test_params: TestParams):
    url = "http://engine-service:8080/disk-stress"
    request_data = {
        "size": test_params.size,
        "duration": test_params.duration,
    }
    response = requests.post(url, json=request_data)
    if response.status_code != 200:
        raise HTTPException(status_code=response.status_code, detail="Failed to start disk stress test")
    return response.text

# Endpoint to handle CPU stress test creation
@app.post("/cpu-stress")
async def start_cpu_stress_test(test_params: TestParams):
    print(f"Starting CPU stress test with intensity: {test_params.intensity}, duration: {test_params.duration}, load: {test_params.load}")
    return {"message": send_cpu_stress_request(test_params)}

# Endpoint to handle memory stress test creation
@app.post("/mem-stress")
async def start_memory_stress_test(test_params: TestParams):
    print(f"Starting memory stress test with size: {test_params.size} MB, duration: {test_params.duration} seconds.")
    return {"message": send_memory_stress_request(test_params)}

# Endpoint to handle disk stress test creation
@app.post("/disk-stress")
async def start_disk_stress_test(test_params: TestParams):
    print(f"Starting disk stress test with size: {test_params.size} MB, duration: {test_params.duration} seconds.")
    return {"message": send_disk_stress_request(test_params)}

# Run the FastAPI app
if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=8081)
