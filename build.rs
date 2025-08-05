// TODO: replace this with meson to automatically compile the blueprint

fn main() {
    glib_build_tools::compile_resources(
        &["data/resources/blueprints"],
        "data/resources/fck-ihk.gresource.xml",
        "compiled.gresources",
    );
}
