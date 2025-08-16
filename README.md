# FITS
Fucking IHK tooling sucks

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
`git clone https://github.com/NoahJeanA/fck-ihk.git`, then
`cd fck-ihk` and finally `cargo build --release` (or `cargo run --release`
to run it directly).


### Project Status - Development on Hold
#### Current Status: Waiting for Platform Access
    Development is currently paused until August 11th, 2025

    Reason for Delay:

    IHK has launched a new platform that requires extensive reverse engineering work.

    We currently do not have access to the new system, which prevents us from continuing development.
#### Timeline
    Current Status: Waiting for platform access

    Expected Resume Date: August 11th, 2025

    Reason: New IHK platform requires significant reverse engineering
