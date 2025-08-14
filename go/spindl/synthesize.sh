#!/bin/bash
# Synthesize FTL CDK apps to spin.toml

if [ $# -eq 0 ]; then
    echo "Usage: ./synthesize.sh <cdk-file.go> [output.toml]"
    echo ""
    echo "Examples:"
    echo "  ./synthesize.sh examples/simple-app.go"
    echo "  ./synthesize.sh examples/auth-app.go spin.toml"
    exit 1
fi

CDK_FILE=$1
OUTPUT_FILE=${2:-spin.toml}

echo "🔨 Synthesizing $CDK_FILE to $OUTPUT_FILE..."

# Run the CDK file and save output
go run "$CDK_FILE" > "$OUTPUT_FILE"

if [ $? -eq 0 ]; then
    echo "✅ Successfully created $OUTPUT_FILE"
    echo ""
    echo "📄 First 20 lines of generated manifest:"
    head -20 "$OUTPUT_FILE"
    echo "..."
    echo ""
    echo "You can now use this with Spin:"
    echo "  spin build"
    echo "  spin up"
else
    echo "❌ Failed to synthesize"
    exit 1
fi