use std::{fs, path::Path};

fn main() {
    println!("cargo:rerun-if-changed=sp1_prover/src");
    
    // SP1 prover binary'sinin konumunu kontrol et
    let target_dir = Path::new("target/elf-compilation/riscv32im-succinct-zkvm-elf/release");
    let target_path = target_dir.join("sp1_prover");
    
    // Eğer binary zaten varsa, yeniden derlemeye gerek yok
    if target_path.exists() {
        println!("SP1 prover binary already exists at {:?}", target_path);
        return;
    }
    
    // Hedef dizini oluştur
    fs::create_dir_all(target_dir).expect("Failed to create target directory");
    
    // SP1 prover binary'sini ara
    let source_path = Path::new("target/release/sp1_prover");
    
    if source_path.exists() {
        println!("Found SP1 prover at {:?}", source_path);
        
        // Dosyayı kopyala
        fs::copy(source_path, &target_path)
            .expect("Failed to copy SP1 prover binary");
        
        println!("Copied SP1 prover to {:?}", target_path);
    } else {
        println!("SP1 prover binary not found at {:?}", source_path);
        println!("Creating an empty file as a fallback");
        
        // Boş bir dosya oluştur
        fs::write(&target_path, &[]).expect("Failed to create empty file");
    }
} 