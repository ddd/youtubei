use std::io::Result;
use std::path::PathBuf;

fn main() -> Result<()> {
    let out_dir = PathBuf::from("pb");

    let proto_files = [
        "innertube/browse.proto",
        "innertube/next.proto",
        "innertube/navigation.proto",
        "innertube/creator.proto",
        "innertube/context.proto",
        "innertube/flag.proto",
    ];

    let proto_paths = [
        PathBuf::from("../youtubei"),
    ];

    // Debug information
    println!("cargo:warning=Current working directory: {:?}", std::env::current_dir().unwrap());
    println!("cargo:warning=Proto paths: {:?}", proto_paths);
    println!("cargo:warning=Proto files: {:?}", proto_files);
    
    // Check if proto files exist
    for proto_file in &proto_files {
        let full_path = proto_paths[0].join(proto_file);
        println!("cargo:warning=Checking proto file: {:?} - exists: {}", full_path, full_path.exists());
    }

    prost_build::Config::new()
        .out_dir(&out_dir)
        .compile_protos(&proto_files, &proto_paths)?;

    Ok(())
}