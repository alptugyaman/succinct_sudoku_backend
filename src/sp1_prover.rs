#![no_main]
use sp1_zkvm::{self, entrypoint, io};
use sudoku_backend_lib::verify_solution;

// SP1 için giriş noktası
#[entrypoint]
fn main() {
    // Orijinal Sudoku tahtasını oku
    let board = io::read::<Vec<Vec<u8>>>();
    
    // Çözümü oku
    let solution = io::read::<Vec<Vec<u8>>>();
    
    // Çözümün doğruluğunu kontrol et
    let is_valid = verify_solution(&board, &solution);
    
    // Sonucu commit et (public output)
    io::commit(&is_valid);
}

// Sudoku çözümünü doğrulayan fonksiyon
fn verify_solution(board: &[Vec<u8>], solution: &[Vec<u8>]) -> bool {
    // 1. Çözümün geçerli bir Sudoku olup olmadığını kontrol et
    if !is_valid_sudoku(solution) {
        return false;
    }
    
    // 2. Çözümün orijinal tahtaya uygun olup olmadığını kontrol et
    for i in 0..9 {
        for j in 0..9 {
            // Orijinal tahtada bir sayı varsa, çözümde de aynı sayı olmalı
            if board[i][j] != 0 && board[i][j] != solution[i][j] {
                return false;
            }
        }
    }
    
    // Tüm kontroller geçildi, çözüm doğru
    true
}

// Sudoku doğrulama fonksiyonu
fn is_valid_sudoku(board: &[Vec<u8>]) -> bool {
    for i in 0..9 {
        let mut row = vec![false; 9];
        let mut col = vec![false; 9];
        let mut box_ = vec![false; 9];

        for j in 0..9 {
            if board[i][j] != 0 {
                if row[(board[i][j] - 1) as usize] {
                    return false;
                }
                row[(board[i][j] - 1) as usize] = true;
            }

            if board[j][i] != 0 {
                if col[(board[j][i] - 1) as usize] {
                    return false;
                }
                col[(board[j][i] - 1) as usize] = true;
            }

            let row_idx = 3 * (i / 3) + j / 3;
            let col_idx = 3 * (i % 3) + j % 3;
            if board[row_idx][col_idx] != 0 {
                if box_[(board[row_idx][col_idx] - 1) as usize] {
                    return false;
                }
                box_[(board[row_idx][col_idx] - 1) as usize] = true;
            }
        }
    }
    true
} 