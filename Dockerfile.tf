# Use the TensorFlow Serving image as the base
FROM tensorflow/serving

# Set the working directory
WORKDIR /model

# Copy the saved model directory into the container
COPY saved_models/ saved_models/

# Define the model server configurations
ENV MODEL_NAME=my_model
ENV MODEL_PATH=/model/saved_models

# Expose the gRPC and REST API ports
EXPOSE 8500
EXPOSE 8501

# Start TensorFlow Serving with the gRPC and REST API
ENTRYPOINT ["tensorflow_model_server"]
CMD ["tensorflow_model_server", "--model_name=my_model", "--model_base_path=/model/saved_models", "--rest_api_port=8501", "--port=8500"]
