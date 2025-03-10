use std::{env, fs, io, path::Path};

fn main() {
    println!("cargo:rerun-if-changed=sp1_prover/src");
    
    // SP1 prover'ı derle
    let status = std::process::Command::new("cargo")
        .args(&["build", "--release", "--manifest-path", "sp1_prover/Cargo.toml"])
        .status()
        .expect("Failed to build SP1 prover");
    
    if !status.success() {
        panic!("Failed to build SP1 prover");
    }
    
    // Hedef dizini oluştur
    let target_dir = Path::new("target/elf-compilation/riscv32im-succinct-zkvm-elf/release");
    fs::create_dir_all(target_dir).expect("Failed to create target directory");
    
    // SP1 prover binary'sini ara
    let source_path = Path::new("sp1_prover/target/release/sp1_prover");
    
    if source_path.exists() {
        println!("Found SP1 prover at {:?}", source_path);
        
        // Hedef dosya yolu
        let target_path = target_dir.join("sp1_prover");
        
        // Dosyayı kopyala
        fs::copy(source_path, &target_path)
            .expect("Failed to copy SP1 prover binary");
        
        println!("Copied SP1 prover to {:?}", target_path);
    } else {
        println!("SP1 prover binary not found at {:?}", source_path);
        println!("Creating an empty file as a fallback");
        
        // Boş bir dosya oluştur
        let target_path = target_dir.join("sp1_prover");
        fs::write(&target_path, &[]).expect("Failed to create empty file");
    }
} 