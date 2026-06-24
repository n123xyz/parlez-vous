use burn_onnx::ModelGen;
use std::env;
use std::path::PathBuf;

fn main() {
    // 1. Get the absolute path to the src-tauri folder
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let onnx_path = PathBuf::from(&manifest_dir).join("models").join("character_model.onnx");
    
    // 2. Put the generated model safely into the target build folder
    let out_dir = env::var("OUT_DIR").unwrap();
    let model_out = PathBuf::from(&out_dir).join("model");

    // Tell cargo to only rebuild if the ONNX file actually changes
    println!("cargo:rerun-if-changed={}", onnx_path.display());

    // 3. Compile the model using absolute paths
    ModelGen::new()
        .input(onnx_path.to_str().unwrap())
        .out_dir(model_out.to_str().unwrap())
        .run_from_script();

    tauri_build::build()
}
