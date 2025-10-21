# FITS
Fucking IHK tooling sucks

### Project Status - Development Proceeding
Development is ongoing and continuing. Things should start to happen more and more
as there is now a somewhat formed idea on what the final software should look like.
We currently aim for a release before 2026, with release candidates starting at the
latest December 1st.

## Installation
### From source

#### Notice
Building from source is **not** what you'll want to do in 99% of cases and should be
considered a last resort if there are no prebuilt packages at all for your
platform (or you are a developer). With that said:

To build FITS from source, you will need to have GTK development libraries, the Rust
toolchain protobuf and sqlite installed. To do so, follow the guide at
<https://rustup.rs> for installing Rust and
<https://www.gtk.org/docs/installations/> for GTK. Sqlite should be installed
through your preferred package manager.

We currently require at least Sqlite 3.35, GTK 4.14 and Rust TODO

Once you are set up with your dependencies, simply run
`git clone https://github.com/NoahJeanA/FITS.git`, then
`cd FITS` and finally `cargo build --release` (or `cargo run --release`
to run it directly).

## API Examples

FITS includes an example program demonstrating how to interact with the FITS API. This example shows how to make health checks and handle various error scenarios.

### Quick Start
```bash
# Run the health check example
cargo run --example api_health_check

# Or use the helper script
./examples/run_health_check.sh

# Enable detailed logging
API_LOG=debug cargo run --example api_health_check
```

See [examples/README.md](examples/README.md) for detailed documentation.
