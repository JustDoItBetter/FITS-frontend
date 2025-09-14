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

To build FITS from source, you will need to have GTK development libraries and the
Rust toolchain installed. To do so, follow the guide at <https://rustup.rs> for
installing Rust and <https://www.gtk.org/docs/installations/> for GTK.

Once you are set up with your dependencies, simply run
`git clone https://github.com/NoahJeanA/FITS.git`, then
`cd FITS` and finally `cargo build --release` (or `cargo run --release`
to run it directly).
