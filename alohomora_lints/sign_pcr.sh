#!/bin/bash

# Function to extract the "foo" part from "foo_hash"
extract_prefix() {
  echo "$1" | sed 's/_src_hash//' | sed 's/_mir_hash//'
}

# Extract the "foo" part from both $2 and $3 (they should have the same prefix)
prefix=$(extract_prefix "$2")

# Sign the first file
ssh-keygen -Y sign -f $1 -n file $2
base64 -i "$2.sig" -o "$2.sig.base"

# Sign the second file
ssh-keygen -Y sign -f $1 -n file $3
base64 -i "$3.sig" -o "$3.sig.base"

dest="${prefix}_SIGNATURE"
# Merge the two signatures into one file with the name "foo_signature"
echo "Wrote Sesame PCR signature to $dest"
echo "$(cat "$2.sig.base")#$(cat "$3.sig.base")" > $dest
