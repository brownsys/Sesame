# Sniffer Examples

This contains three examples of unsafe code used to attempt to circumvent the bbox protections.

### sniffer1

This is an inline example (no helper functions or dependencies), it uses unsafe code to
dump the byte-content of a BBox.

This does not actually leak the data, due to pcon indrection.

### sniffer2

This example uses `unsafelib`, which uses the same code as sniffer1 to log the byte content of
any type, not just BBoxs. 

`unsafelib` also provides a second api to circumvent pcon indirection, by maliciously derefering
the nested pointer and xoring it with the secret.

### sniffer3

The apikey/check endpoint from websubmit extracted out of rocket and simplified.

The endpoint checks the database to see if the input apikey exists or not.

The endpoint also uses `unsafelib` to log the sensitive apikey.
