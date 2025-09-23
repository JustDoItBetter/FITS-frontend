// TODO: replace this with meson to automatically compile the blueprint

fn main() {
    glib_build_tools::compile_resources(
        &["data/resources/blueprints", "data/resources"],
        "data/fits.gresource.xml",
        "compiled.gresources",
    );

    protobuf_codegen::CodeGen::new()
        .inputs(["report.proto"])
        .include("data/resources/protobuf")
        .generate_and_compile()
        .expect("Protobuf compile failed: ")
}
