# Examples of Privacy Critical Region Signing

Enter an example directory, e.g., avoid_false_positives/, then run `cargo dylint --lib alohomora_lints`.

## Overview of cases
`avoid_false_positives`

Adding whitespace or comments won't invalidate a signature.

`blank_sign_illegal`

Blank signatures in any field will fail.

`copy_sign_illegal`

Copying a function review signature from one PCR to another will fail.

Copying a function review signature to the dependency review signature field will fail (and vice versa).

`dep_change_illegal`

Changing dependencies that are used in the closure will cause all signatures to fail. Dependency changes are indicated by version bumps. In this example, bumping `dependency` from 0.1.0 -> 0.2.0 causes the signatures to fail.

`rec_change_illegal`

Changing an in-crate function that is transitively called in the signed closure will fail. In this example, function `third::grandchild` has a breaking change.

KNOWN BUG: `const_change_illegal`

Changing the values of a constant or static should invalidate the signature, but currently does not.
