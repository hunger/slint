#!/usr/bin/bash

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

# Define the source directory and target directory
SOURCE_DIR="$1"
TARGET_DIR="${SCRIPT_DIR}/corpus/${2}"

# Create the target directory if it doesn't exist
test -d "${TARGET_DIR}" || exit 2;

# Initialize a counter
counter=${3:-1}

# Find and copy files, numbering them
find "$SOURCE_DIR" -type f -name "*.slint" | while read -r file; do
    cp "$file" "$TARGET_DIR/${counter}.slint"
    ((counter++))
done
