#!/bin/bash

# s.t. $1 is private key and $2 is Cargo.lock_hash
ssh-keygen -Y sign -f $1 -n file $2
base64 "$2.sig" > "$2.sig.base"
echo ""
echo "Signature in base64"
cat "$2.sig.base"  | tr -d '\n'
echo ""
