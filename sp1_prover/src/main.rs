#![no_main]
// Update the import to use the new API structure
sp1_zkvm::entrypoint!(verify_sudoku);

// SP1 içinde çalışacak Sudoku doğrulama fonksiyonu
fn verify_sudoku() {
    // Giriş verilerini oku
    let initial_board: Vec<Vec<u8>> = sp1_zkvm::io::read();
    let solution: Vec<Vec<u8>> = sp1_zkvm::io::read();
    
    // Çözümün doğruluğunu kontrol et
    let is_valid = verify_replay(&initial_board, &solution);
    
    // Sonucu yaz
    sp1_zkvm::io::commit(&is_valid);
}

// Sudoku çözümünün başlangıç tahtasına uygun olup olmadığını kontrol eden fonksiyon
fn verify_replay(initial_board: &[Vec<u8>], solution: &[Vec<u8>]) -> bool {
    // 1. Çözümün geçerli bir Sudoku olup olmadığını kontrol et
    if !is_valid_solution(solution) {
        return false;
    }
    
    // 2. Çözümün başlangıç tahtasına uygun olup olmadığını kontrol et
    for i in 0..9 {
        for j in 0..9 {
            // Başlangıç tahtasında bir sayı varsa, çözümde de aynı sayı olmalı
            if initial_board[i][j] != 0 && initial_board[i][j] != solution[i][j] {
                return false;
            }
        }
    }
    
    // Tüm kontroller geçildi, çözüm doğru
    true
}

// Sudoku çözümünün geçerli olup olmadığını kontrol eden fonksiyon
fn is_valid_solution(board: &[Vec<u8>]) -> bool {
    // Tüm hücrelerin doldurulmuş olup olmadığını kontrol et
    for row in board {
        for &cell in row {
            if cell == 0 {
                return false; // Boş hücre var, çözüm tamamlanmamış
            }
        }
    }

    // Satırları kontrol et
    for row in board {
        let mut seen = [false; 10]; // 1-9 için, 0. indeksi kullanmıyoruz
        for &num in row {
            if num < 1 || num > 9 || seen[num as usize] {
                return false; // Geçersiz veya tekrarlanan rakam
            }
            seen[num as usize] = true;
        }
    }

    // Sütunları kontrol et
    for col in 0..9 {
        let mut seen = [false; 10]; // 1-9 için, 0. indeksi kullanmıyoruz
        for row in 0..9 {
            let num = board[row][col];
            if seen[num as usize] {
                return false; // Tekrarlanan rakam
            }
            seen[num as usize] = true;
        }
    }

    // 3x3 kutuları kontrol et
    for box_row in 0..3 {
        for box_col in 0..3 {
            let mut seen = [false; 10]; // 1-9 için, 0. indeksi kullanmıyoruz
            for i in 0..3 {
                for j in 0..3 {
                    let num = board[box_row * 3 + i][box_col * 3 + j];
                    if seen[num as usize] {
                        return false; // Tekrarlanan rakam
                    }
                    seen[num as usize] = true;
                }
            }
        }
    }

    true // Tüm kontroller geçildi, çözüm geçerli
} 