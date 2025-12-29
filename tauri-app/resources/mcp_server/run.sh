#!/bin/bash
# Wrapper script to run MCP server with venv

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$DIR"

# Activate venv
source .venv/bin/activate

# Run main.py
python main.py
