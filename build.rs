use anyhow::Result;

fn main() -> Result<()> {
    println!("cargo:rustc-env={}={}", "RUST_LOG", "DEBUG");

    tonic_build::compile_protos("proto/chat.proto")?;
    Ok(())
}
