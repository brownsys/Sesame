# websubmit-rs: a simple class submission system

This is a fork for websubmit-rs, a web application for collecting student homework
submissions, written using [Rocket](https://rocket.rs) for a MySQL backend.

To run it, you need to run a MySQL server deployment.
Then you can run the web application, which will automatically connect
to MySQL database `myclass`:
```
websubmit-rs$ cargo run --release -- -i myclass
```
To create and initialize the database, set the `prime` variable in the configuration
file (see below).

The web interface will be served on `localhost:8000`. Note that the
templates included in this repository are very basic; in practice, you
will want to customize the files in `templates`.

By default, the application will read configuration file `sample-config.toml`,
but a real deployment will specify a custom config (`-c myconfig.toml`).
Configuration files are TOML files with the following format:
```
# short class ID (human readable)
class = "CSCI 2390"
# list of staff email addresses (these users' API keys get admin access)
staff = ["malte@cs.brown.edu"]
# custom template directory
template_dir = "/path/to/templates"
# custom resource directory (e.g., for images, CSS, JS)
resource_dir = "/path/to/resources"
# a secret that will be hashed into user's API keys to make them unforgeable
secret = "SECRET"
# whether to send emails (set to false for development)
send_emails = false
# whether to reset the db (set to false for production)
prime = true
```

If you omit `--release`, the web app will produce additional
debugging output.

