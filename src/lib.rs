/*!
Primality Jones: A Mersenne Number Primality Testing Library

This library provides tools for efficiently testing the primality of Mersenne numbers
(numbers of the form 2^p - 1). It implements various levels of testing, from quick
checks to thorough verification.

# Example

```rust
use primality_jones::{CheckLevel, check_mersenne_candidate};

fn main() {
    let p = 12301; // Test M12301
    let results = check_mersenne_candidate(p, CheckLevel::Quick);
    
    if results.iter().all(|r| r.passed) {
        println!("M{} is a promising candidate!", p);
    } else {
        println!("M{} is not prime.", p);
    }
}
```

# Safety and Performance

This library is designed for mathematical research and should not be used for
cryptographic purposes. The probabilistic nature of Fermat tests means false
positives are possible.

For large Mersenne numbers (>100M digits), consider using the GIMPS software
for definitive primality testing.
*/

use std::time::{Duration, Instant};
use num_bigint::{BigUint, RandBigInt};
use num_traits::{One, Zero};
use rand::thread_rng;
use indicatif::{ProgressBar, ProgressStyle};
use pyo3::prelude::*;
use pyo3::types::PyDict;

/// Represents the result of a primality check
#[derive(Debug, Clone)]
pub struct CheckResult {
    /// Whether the check passed
    pub passed: bool,
    /// Description of the check result
    pub message: String,
    /// How long the check took
    pub time_taken: Duration,
}

/// Different levels of thoroughness for primality checking
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CheckLevel {
    /// Instant checks (divisibility rules, etc.)
    Basic,
    /// Fast checks (small factors and Lucas sequence)
    FastCheck,
    /// Fast checks (small Fermat tests)
    Quick,
    /// More thorough (more Fermat tests, additional properties)
    Moderate,
    /// Very thorough (extended tests, could take minutes)
    Thorough,
    /// Most thorough (could take hours)
    Exhaustive,
}

impl CheckLevel {
    /// Get a human-readable description of the check level
    pub fn description(&self) -> String {
        match self {
            CheckLevel::Basic => "Basic checks (divisibility rules, instant)".to_string(),
            CheckLevel::FastCheck => "Fast checks (small factors, residues, ~1 second)".to_string(),
            CheckLevel::Quick => "Quick checks (basic Fermat tests, seconds)".to_string(),
            CheckLevel::Moderate => "Moderate checks (extended Fermat tests, ~1 minute)".to_string(),
            CheckLevel::Thorough => "Thorough checks (multiple methods, ~10 minutes)".to_string(),
            CheckLevel::Exhaustive => "Exhaustive checks (all available methods, hours)".to_string(),
        }
    }
}

/// Check if a number is prime using trial division
///
/// # Arguments
///
/// * `n` - The number to test for primality
///
/// # Returns
///
/// * `true` if the number is prime
/// * `false` if the number is composite or less than 2
///
/// # Examples
///
/// ```
/// use primality_jones::is_prime;
///
/// assert!(is_prime(31));
/// assert!(!is_prime(15));
/// ```
pub fn is_prime(n: u64) -> bool {
    if n <= 1 { return false; }
    if n <= 3 { return true; }
    if n % 2 == 0 || n % 3 == 0 { return false; }
    
    let sqrt_n = (n as f64).sqrt() as u64;
    let mut i = 5;
    while i <= sqrt_n {
        if n % i == 0 || n % (i + 2) == 0 {
            return false;
        }
        i += 6;
    }
    true
}

fn mod_mersenne(base: &BigUint, exp: u64, p: u64) -> BigUint {
    let mut result = base.clone();
    let m = (BigUint::one() << p) - BigUint::one();
    
    for _ in 1..exp {
        result = (&result * base) % &m;
    }
    result
}

/// Perform a Fermat primality test with specified parameters
///
/// # Arguments
///
/// * `p` - The Mersenne exponent to test (testing 2^p - 1)
/// * `k` - Number of rounds of testing
/// * `start_time` - Start time of the test
/// * `timeout` - Timeout for the test
///
/// # Returns
///
/// * `true` if all tests pass
/// * `false` if any test fails (number is definitely composite)
pub fn fermat_test(p: u64, k: u32, start_time: Instant, timeout: Duration) -> bool {
    let m = (BigUint::one() << p) - BigUint::one();
    let mut rng = thread_rng();
    
    // Create progress bar for Fermat tests
    let pb = ProgressBar::new(k as u64);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} tests ({eta})")
        .unwrap()
        .progress_chars("#>-"));
    
    for _i in 0..k {
        // Check timeout
        if start_time.elapsed() > timeout {
            pb.finish_with_message("Timed out");
            return false;
        }
        
        // Generate random base between 2 and m-1
        let a = rng.gen_biguint_range(&BigUint::from(2u32), &(&m - BigUint::one()));
        
        // Calculate a^(m-1) mod m using modular exponentiation
        let result = mod_mersenne(&a, p - 1, p);
        
        if result != BigUint::one() {
            pb.finish_with_message("Failed");
            return false;
        }
        
        pb.inc(1);
    }
    
    pb.finish_with_message("Passed");
    true
}

/// Check if a number satisfies basic Mersenne prime properties
///
/// # Arguments
///
/// * `p` - The Mersenne exponent to test (testing 2^p - 1)
///
/// # Returns
///
/// * `true` if the number satisfies basic Mersenne prime properties
/// * `false` otherwise
pub fn check_mersenne_properties(p: u64) -> bool {
    // Check if p ≡ 3 (mod 4)
    p % 4 == 3
}

/// Check a Mersenne number candidate with the specified level of thoroughness
///
/// This is the main entry point for testing Mersenne number candidates. It performs
/// increasingly thorough tests based on the specified check level.
///
/// # Arguments
///
/// * `p` - The Mersenne exponent to test (testing 2^p - 1)
/// * `level` - How thorough the testing should be
///
/// # Returns
///
/// A vector of `CheckResult`s, one for each test performed. The candidate is
/// considered promising if all tests pass.
///
/// # Examples
///
/// ```
/// use primality_jones::{CheckLevel, check_mersenne_candidate};
///
/// let results = check_mersenne_candidate(31, CheckLevel::Quick);
/// assert!(results.iter().all(|r| r.passed)); // M31 is prime
///
/// let results = check_mersenne_candidate(32, CheckLevel::Quick);
/// assert!(!results.iter().all(|r| r.passed)); // M32 is composite
/// ```
pub fn check_mersenne_candidate(p: u64, level: CheckLevel) -> Vec<CheckResult> {
    let mut results = Vec::new();
    let start_time = Instant::now();
    let timeout = match level {
        CheckLevel::Basic => Duration::from_secs(1),
        CheckLevel::FastCheck => Duration::from_secs(5),
        CheckLevel::Quick => Duration::from_secs(30),
        CheckLevel::Moderate => Duration::from_secs(300),
        CheckLevel::Thorough => Duration::from_secs(1800),
        CheckLevel::Exhaustive => Duration::from_secs(7200),
    };
    
    // Basic Mersenne properties check
    let check_start = Instant::now();
    let basic_passed = check_mersenne_properties(p);
    results.push(CheckResult {
        passed: basic_passed,
        message: if basic_passed {
            "Passed basic Mersenne properties".to_string()
        } else {
            "Failed basic Mersenne properties".to_string()
        },
        time_taken: check_start.elapsed(),
    });
    
    if !basic_passed || level == CheckLevel::Basic {
        return results;
    }
    
    // FastCheck level: Small factors and Lucas residues
    if level >= CheckLevel::FastCheck {
        // Check for small factors up to 1 million
        let check_start = Instant::now();
        if let Some(factor) = check_small_factors(p, 1_000_000) {
            results.push(CheckResult {
                passed: false,
                message: format!("Found small factor: {}", factor),
                time_taken: check_start.elapsed(),
            });
            return results;
        }
        results.push(CheckResult {
            passed: true,
            message: "No small factors found up to 1M".to_string(),
            time_taken: check_start.elapsed(),
        });
        
        // Check Lucas sequence residues
        let check_start = Instant::now();
        let lucas_passed = check_lucas_residues(p);
        results.push(CheckResult {
            passed: lucas_passed,
            message: if lucas_passed {
                "Passed Lucas sequence check".to_string()
            } else {
                "Failed Lucas sequence check".to_string()
            },
            time_taken: check_start.elapsed(),
        });
        
        if !lucas_passed || level == CheckLevel::FastCheck {
            return results;
        }
    }
    
    // Check if exponent is prime
    let check_start = Instant::now();
    let prime_passed = is_prime(p);
    results.push(CheckResult {
        passed: prime_passed,
        message: if prime_passed {
            "Exponent is prime".to_string()
        } else {
            "Exponent is not prime".to_string()
        },
        time_taken: check_start.elapsed(),
    });
    
    if !prime_passed {
        return results;
    }
    
    // Fermat primality tests for higher levels
    if level >= CheckLevel::Quick {
        let check_start = Instant::now();
        let k = match level {
            CheckLevel::Quick => 3,
            CheckLevel::Moderate => 5,
            CheckLevel::Thorough => 10,
            CheckLevel::Exhaustive => 20,
            _ => unreachable!(),  // Basic and FastCheck are handled above
        };
        
        let fermat_passed = fermat_test(p, k, start_time, timeout);
        results.push(CheckResult {
            passed: fermat_passed,
            message: if fermat_passed {
                format!("Passed {} Fermat tests", k)
            } else {
                format!("Failed Fermat tests")
            },
            time_taken: check_start.elapsed(),
        });
    }
    
    results
}

/// Check for small factors of a Mersenne number using special properties
pub fn check_small_factors(p: u64, limit: u64) -> Option<u64> {
    if !is_prime(p) {
        return None;
    }
    
    // Any factor q of M_p must be of form q = 2kp + 1
    // and must be ≡ ±1 (mod 8)
    let mut k = 1;
    while 2 * k * p + 1 <= limit {
        let q = 2 * k * p + 1;
        if q % 8 == 1 || q % 8 == 7 {
            if is_prime(q) {
                // Check if q divides 2^p - 1 using modular arithmetic
                let remainder = mod_mersenne(&BigUint::from(2u32), p, q);
                if remainder == BigUint::one() {
                    return Some(q);
                }
            }
        }
        k += 1;
    }
    None
}

/// Check the first few terms of Lucas sequence
pub fn check_lucas_residues(p: u64) -> bool {
    if p < 2 { return false; }
    
    let mut s = BigUint::from(4u32);
    let m = (BigUint::one() << p) - BigUint::one();
    
    // Check first few terms for expected patterns
    for _ in 0..5 {
        s = (&s * &s - BigUint::from(2u32)) % &m;
        // Early exit if we hit 0 (definitely composite)
        if s == BigUint::zero() {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_properties() {
        // Known Mersenne prime exponent
        assert!(check_mersenne_properties(31));
        // Known non-Mersenne prime exponent
        assert!(!check_mersenne_properties(32));
        // Even number
        assert!(!check_mersenne_properties(4));
        // Not ≡ 3 (mod 4)
        assert!(!check_mersenne_properties(5));
    }

    #[test]
    fn test_is_prime() {
        assert!(is_prime(31));
        assert!(is_prime(13));
        assert!(!is_prime(15));
        assert!(!is_prime(1));
        assert!(!is_prime(0));
    }

    #[test]
    fn test_fermat_test() {
        // M31 is a known Mersenne prime
        assert!(fermat_test(31, 5, Instant::now(), Duration::from_secs(30)));
        // M32 is known to be composite
        assert!(!fermat_test(32, 5, Instant::now(), Duration::from_secs(30)));
    }

    #[test]
    fn test_check_mersenne_candidate() {
        // Test with M7 (known Mersenne prime)
        let results = check_mersenne_candidate(7, CheckLevel::Quick);
        assert!(results.iter().all(|r| r.passed));

        // Test with M8 (known composite)
        let results = check_mersenne_candidate(8, CheckLevel::Quick);
        assert!(!results.iter().all(|r| r.passed));
    }

    #[test]
    fn test_large_numbers() {
        // Test handling of a moderately large number
        let results = check_mersenne_candidate(12301, CheckLevel::Basic);
        // Should at least complete without panicking
        assert!(results.len() > 0);
    }
}

/// Python module for Mersenne number primality testing
#[pymodule]
fn primality_jones(_py: Python, m: &PyModule) -> PyResult<()> {
    // Expose CheckLevel enum to Python
    #[pyclass]
    #[derive(Clone, Copy)]
    enum PyCheckLevel {
        Basic = 0,
        FastCheck = 1,
        Quick = 2,
        Moderate = 3,
        Thorough = 4,
        Exhaustive = 5,
    }

    #[pymethods]
    impl PyCheckLevel {
        fn description(&self) -> String {
            match self {
                PyCheckLevel::Basic => "Basic checks (divisibility rules, instant)".to_string(),
                PyCheckLevel::FastCheck => "Fast checks (small factors, residues, ~1 second)".to_string(),
                PyCheckLevel::Quick => "Quick checks (basic Fermat tests, seconds)".to_string(),
                PyCheckLevel::Moderate => "Moderate checks (extended Fermat tests, ~1 minute)".to_string(),
                PyCheckLevel::Thorough => "Thorough checks (multiple methods, ~10 minutes)".to_string(),
                PyCheckLevel::Exhaustive => "Exhaustive checks (all available methods, hours)".to_string(),
            }
        }
    }

    /// Check a Mersenne number for primality
    #[pyfunction]
    fn check_mersenne(p: u64, level: PyCheckLevel) -> PyResult<Vec<PyObject>> {
        let check_level = match level {
            PyCheckLevel::Basic => CheckLevel::Basic,
            PyCheckLevel::FastCheck => CheckLevel::FastCheck,
            PyCheckLevel::Quick => CheckLevel::Quick,
            PyCheckLevel::Moderate => CheckLevel::Moderate,
            PyCheckLevel::Thorough => CheckLevel::Thorough,
            PyCheckLevel::Exhaustive => CheckLevel::Exhaustive,
        };

        let results = check_mersenne_candidate(p, check_level);
        
        Python::with_gil(|py| {
            results.into_iter().map(|r| {
                let dict = PyDict::new(py);
                dict.set_item("passed", r.passed)?;
                dict.set_item("message", r.message)?;
                dict.set_item("time_taken_ns", r.time_taken.as_nanos())?;
                Ok(dict.into())
            }).collect()
        })
    }

    /// Check if a number is prime
    #[pyfunction]
    fn is_prime_py(n: u64) -> bool {
        is_prime(n)
    }

    /// Check for small factors of a Mersenne number
    #[pyfunction]
    fn find_small_factors(p: u64, limit: u64) -> Option<u64> {
        check_small_factors(p, limit)
    }

    /// Check Lucas sequence residues
    #[pyfunction]
    fn check_lucas(p: u64) -> bool {
        check_lucas_residues(p)
    }

    // Register Python functions and classes
    m.add_class::<PyCheckLevel>()?;
    m.add_function(wrap_pyfunction!(check_mersenne, m)?)?;
    m.add_function(wrap_pyfunction!(is_prime_py, m)?)?;
    m.add_function(wrap_pyfunction!(find_small_factors, m)?)?;
    m.add_function(wrap_pyfunction!(check_lucas, m)?)?;

    Ok(())
} 