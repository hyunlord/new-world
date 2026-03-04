//! Math utilities for simulation systems.
//!
//! Provides Cholesky decomposition for correlated random variable generation.

/// Cholesky decomposition: decomposes a symmetric positive-definite matrix A into L × L^T.
///
/// # Arguments
/// - `matrix`: n×n symmetric positive-definite matrix (as slice of rows)
///
/// # Returns
/// Lower triangular matrix L such that L × L^T ≈ matrix
///
/// # Reference
/// Used for correlated HEXACO personality axis generation (Ashton & Lee 2009).
pub fn cholesky_decompose(matrix: &[Vec<f64>]) -> Vec<Vec<f64>> {
    let n = matrix.len();
    let mut l = vec![vec![0.0f64; n]; n];
    for i in 0..n {
        for j in 0..=i {
            let mut sum = 0.0;
            for k in 0..j {
                sum += l[i][k] * l[j][k];
            }
            if i == j {
                l[i][j] = (matrix[i][i] - sum).max(0.0).sqrt();
            } else {
                l[i][j] = if l[j][j].abs() > 1e-12 {
                    (matrix[i][j] - sum) / l[j][j]
                } else {
                    0.0
                };
            }
        }
    }
    l
}

/// Multiply lower triangular matrix L by vector z: result = L × z
///
/// Used after `cholesky_decompose` to transform independent z-scores into correlated ones.
pub fn cholesky_multiply(l: &[Vec<f64>], z: &[f64]) -> Vec<f64> {
    let n = l.len();
    let mut result = vec![0.0f64; n];
    for i in 0..n {
        for j in 0..=i {
            result[i] += l[i][j] * z[j];
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cholesky_identity() {
        let n = 3;
        let identity: Vec<Vec<f64>> = (0..n)
            .map(|i| (0..n).map(|j| if i == j { 1.0 } else { 0.0 }).collect())
            .collect();
        let l = cholesky_decompose(&identity);
        for i in 0..n {
            for j in 0..n {
                let expected = if i == j { 1.0 } else { 0.0 };
                assert!(
                    (l[i][j] - expected).abs() < 1e-10,
                    "L[{i}][{j}] = {} expected {expected}",
                    l[i][j]
                );
            }
        }
    }

    #[test]
    fn test_cholesky_hexaco() {
        let matrix = vec![
            vec![1.00, 0.12, -0.11, 0.26, 0.18, 0.21],
            vec![0.12, 1.00, -0.13, -0.08, 0.15, -0.10],
            vec![-0.11, -0.13, 1.00, 0.05, 0.10, 0.08],
            vec![0.26, -0.08, 0.05, 1.00, 0.01, 0.03],
            vec![0.18, 0.15, 0.10, 0.01, 1.00, 0.03],
            vec![0.21, -0.10, 0.08, 0.03, 0.03, 1.00],
        ];
        let n = matrix.len();
        let l = cholesky_decompose(&matrix);

        // Reconstruct L × L^T and compare to original
        for i in 0..n {
            for j in 0..n {
                let mut reconstructed = 0.0f64;
                for k in 0..n {
                    reconstructed += l[i][k] * l[j][k];
                }
                assert!(
                    (reconstructed - matrix[i][j]).abs() < 1e-10,
                    "Reconstruction error at [{i}][{j}]: got {reconstructed}, expected {}",
                    matrix[i][j]
                );
            }
        }
    }

    #[test]
    fn test_cholesky_multiply_basic() {
        // 2×2 identity: L is identity, multiply by unit vector
        let matrix = vec![vec![1.0, 0.0], vec![0.0, 1.0]];
        let l = cholesky_decompose(&matrix);
        let z = vec![1.0, 0.0];
        let result = cholesky_multiply(&l, &z);
        assert!((result[0] - 1.0).abs() < 1e-10, "result[0] = {}", result[0]);
        assert!((result[1] - 0.0).abs() < 1e-10, "result[1] = {}", result[1]);

        // Non-trivial: correlated 2×2 matrix
        let corr = vec![vec![1.0, 0.5], vec![0.5, 1.0]];
        let l2 = cholesky_decompose(&corr);
        let z2 = vec![1.0, 1.0];
        let result2 = cholesky_multiply(&l2, &z2);
        // result must be non-zero
        assert!(result2[0].abs() > 1e-10 || result2[1].abs() > 1e-10);
    }
}
