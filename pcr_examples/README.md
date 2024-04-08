# Examples of Privacy Critical Region Signing

## Overview of cases

`fn_calls_legal`

A PCR with valid author and function reviewer signatures on the hash of the closure MIR + dependency reviewer signature on the hash of the Cargo.lock. 

`blank_sign_illegal`

Blank signatures will fail. 

`copy_sign_illegal`

Copying a function review signature from one PCR to another will fail.

Copying a function review signature to the dependency review signature field will fail (and vice versa).

`dep_change_illegal`

After signing the Cargo.lock of a crate, changing dependencies (as indicated by version change) will fail.

`rec_change_illegal`

Changing a function (imported from a different module in the same crate) that is called in the signed closure will fail. 

TODO: `const_change_illegal`

Changing the values of a constant or static should invalidate the signature, but currently does not.