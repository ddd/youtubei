use std::env;
use std::io::Result;

fn main() -> Result<()> {
    let proto_files = [
        "innertube/browse.proto",
        "innertube/next.proto",
        "innertube/navigation.proto",
        "innertube/creator.proto",
        "innertube/context.proto",
        "innertube/flag.proto",
    ];

    let proto_paths = ["."];

    // Tell Cargo to rerun this build script if any of the proto files change
    for proto_file in &proto_files {
        println!("cargo:rerun-if-changed={}", proto_file);
    }

    // Generate protobuf files in OUT_DIR
    prost_build::Config::new()
        .out_dir(env::var("OUT_DIR").unwrap())
        .compile_protos(&proto_files, &proto_paths)?;

    Ok(())
}