#!/bin/bash

# make diff directory
mkdir -p diff/src

# get diffs for files that existing in pre-alohomora branch
FAILED_FILES=""
for filename in src/*; do
    git cat-file -e pre-alohomora:"$filename" 2> /dev/null

    if [[ $? -eq 0 ]]; then
        touch diff/"${filename%.rs}.diff"
        git diff -w pre-alohomora:"$filename" "$filename" > diff/"${filename%.rs}.diff"
        echo "created diff file for \033[0;32m$filename\033[0m"
    else
        FAILED_FILES+="\033[0;31m$filename\033[0m, "
    fi
done

# print out list of ones that don't
if [ "$FAILED_FILES" != "" ]; then
    f=$(echo "$FAILED_FILES" | rev | cut -c7- | rev)
    echo "[!] some files didn't exist before alohomora: ${f}"
fi