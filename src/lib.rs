use serde::{Deserialize, Serialize};

// Sudoku tahtası ve çözümü için veri modelleri
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SudokuBoard {
    pub board: Vec<Vec<u8>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SudokuSolution {
    pub solution: Vec<Vec<u8>>,
}

// ZKP için giriş ve çıkış modelleri
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofInput {
    pub board: Vec<Vec<u8>>,
    pub solution: Vec<Vec<u8>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofResponse {
    pub public_values: String,
    pub proof: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobResponse {
    pub job_id: String,
    pub status: String,
    pub result: Option<ProofResponse>,
    pub error: Option<String>,
}

// Sudoku doğrulama fonksiyonları
pub fn is_valid_sudoku(board: &[Vec<u8>]) -> bool {
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

pub fn verify_solution(board: &[Vec<u8>], solution: &[Vec<u8>]) -> bool {
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