import requests
import json
import numpy as np

# Input values
values = [30.0, 1, 4, 0, 617, 554, 124, 5, 0, 0, 1, 37, 42, 33, 1,200]

# Create a NumPy array from the input values
data = np.array(values, dtype=np.float32)

# Reshape the data to a 2D tensor (assuming your model expects a single instance)
data = data.reshape(1, -1)  # (1, 12) for a single instance with 12 features

# Prepare the request payload
payload = {
    "instances": data.tolist()
}

# Send a prediction request to the TensorFlow Serving server
url = "http://localhost:8605/v1/models/cricket_model:predict"
headers = {"content-type": "application/json"}
response = requests.post(url, data=json.dumps(payload), headers=headers)

# Check if the request was successful
if response.status_code == 200:
    # Get the prediction result
    prediction = response.json()["predictions"][0]
    print(f"Prediction: {prediction}")
else:
    print(f"Error: {response.status_code} - {response.text}")
