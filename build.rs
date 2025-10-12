use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let protos = ["./src/protos/server.proto"];

    for p in &protos {
        println!("cargo:rerun-if-changed={}", p);
    }
    println!("cargo:rerun-if-changed=build.rs");

    for p in &protos {
        tonic_prost_build::compile_protos(p)?;
    }

    Ok(())
}
