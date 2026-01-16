#!/bin/bash

# Client SDK Generation Script for StarkMate API
# This script generates client SDKs for TypeScript, Python, and Rust

# Set variables
API_ENDPOINT="http://localhost:8080/api/docs/openapi.json"
OUTPUT_DIR="./generated-clients"
OPENAPI_JSON="./openapi.json"

# Create output directory
mkdir -p $OUTPUT_DIR
mkdir -p $OUTPUT_DIR/typescript
mkdir -p $OUTPUT_DIR/python
mkdir -p $OUTPUT_DIR/rust

# Step 1: Download the OpenAPI specification
echo "Downloading OpenAPI specification from $API_ENDPOINT..."
curl -s $API_ENDPOINT -o $OPENAPI_JSON

# Check if OpenAPI spec was downloaded successfully
if [ ! -f $OPENAPI_JSON ]; then
    echo "Error: Could not download OpenAPI specification."
    echo "Make sure the StarkMate API server is running at localhost:8080"
    exit 1
fi

# Step 2: Generate TypeScript client
echo "Generating TypeScript client..."
npx @openapitools/openapi-generator-cli generate \
    -i $OPENAPI_JSON \
    -g typescript-axios \
    -o $OUTPUT_DIR/typescript \
    --additional-properties=npmName=@xlmate/api-client,npmVersion=1.0.0,supportsES6=true,withSeparateModelsAndApi=true

# Step 3: Generate Python client
echo "Generating Python client..."
npx @openapitools/openapi-generator-cli generate \
    -i $OPENAPI_JSON \
    -g python \
    -o $OUTPUT_DIR/python \
    --additional-properties=packageName=xlmate_client,packageVersion=1.0.0

# Step 4: Generate Rust client
echo "Generating Rust client..."
npx @openapitools/openapi-generator-cli generate \
    -i $OPENAPI_JSON \
    -g rust \
    -o $OUTPUT_DIR/rust \
    --additional-properties=packageName=xlmate-client,packageVersion=1.0.0

# Step 5: Clean up
echo "Cleaning up..."
rm $OPENAPI_JSON

echo "Client SDK generation complete!"
echo "Generated clients can be found in:"
echo "  - TypeScript: $OUTPUT_DIR/typescript"
echo "  - Python: $OUTPUT_DIR/python"
echo "  - Rust: $OUTPUT_DIR/rust"
echo ""
echo "To use the TypeScript client:"
echo "  cd $OUTPUT_DIR/typescript && npm install && npm run build"
echo ""
echo "To use the Python client:"
echo "  cd $OUTPUT_DIR/python && pip install -e ."
echo ""
echo "To use the Rust client:"
echo "  cd $OUTPUT_DIR/rust && cargo build"
