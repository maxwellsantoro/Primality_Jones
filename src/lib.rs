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

use std::time::Duration;
use num_bigint::BigUint;
use num_traits::One;
use rand::Rng;
use rand::thread_rng;

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
    pub fn description(&self) -> &str {
        match self {
            CheckLevel::Basic => "Basic checks (divisibility rules, instant)",
            CheckLevel::Quick => "Quick checks (basic Fermat tests, seconds)",
            CheckLevel::Moderate => "Moderate checks (extended Fermat tests, ~1 minute)",
            CheckLevel::Thorough => "Thorough checks (multiple methods, ~10 minutes)",
            CheckLevel::Exhaustive => "Exhaustive checks (all available methods, hours)"
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
    if n < 2 {
        return false;
    }

    let limit = (n as f64).sqrt() as u64 + 1;
    for i in 2..=limit {
        if n % i == 0 {
            return false;
        }
    }
    true
}

/// Perform a Fermat primality test with specified parameters
///
/// # Arguments
///
/// * `p` - The Mersenne exponent to test (testing 2^p - 1)
/// * `k` - Number of rounds of testing
/// * `max_base` - Maximum base to use in testing
///
/// # Returns
///
/// * `true` if all tests pass
/// * `false` if any test fails (number is definitely composite)
pub fn fermat_test(p: u64, k: u32, max_base: u32) -> bool {
    let mut rng = thread_rng();
    
    // For Mersenne numbers M_p = 2^p - 1, we can use Fermat's little theorem
    let m = (BigUint::from(1u32) << p) - BigUint::one();
    
    for _ in 0..k {
        let a = rng.gen_range(2..=max_base);
        let a_big = BigUint::from(a);
        
        // If m is prime, then a^(m-1) ≡ 1 (mod m)
        let result = a_big.modpow(&(&m - BigUint::one()), &m);
        
        if result != BigUint::one() {
            return false;
        }
    }
    
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
    // Must be odd
    if p % 2 == 0 {
        return false;
    }
    
    // Must be ≡ 3 (mod 4) for large Mersenne primes
    if p % 4 != 3 {
        return false;
    }
    
    true
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
    use std::time::Instant;
    let mut results = Vec::new();
    
    // Level 1: Basic checks
    let start = Instant::now();
    if !check_mersenne_properties(p) {
        results.push(CheckResult {
            passed: false,
            message: format!("Failed basic Mersenne properties"),
            time_taken: start.elapsed(),
        });
        return results;
    }
    results.push(CheckResult {
        passed: true,
        message: "Passed basic Mersenne properties".to_string(),
        time_taken: start.elapsed(),
    });
    
    if level == CheckLevel::Basic {
        return results;
    }
    
    // Level 2: Quick checks
    let start = Instant::now();
    if !is_prime(p) {
        results.push(CheckResult {
            passed: false,
            message: format!("Exponent {} is not prime", p),
            time_taken: start.elapsed(),
        });
        return results;
    }
    results.push(CheckResult {
        passed: true,
        message: "Exponent is prime".to_string(),
        time_taken: start.elapsed(),
    });
    
    let start = Instant::now();
    if !fermat_test(p, 2, 1000) {
        results.push(CheckResult {
            passed: false,
            message: "Failed quick Fermat tests".to_string(),
            time_taken: start.elapsed(),
        });
        return results;
    }
    results.push(CheckResult {
        passed: true,
        message: "Passed quick Fermat tests".to_string(),
        time_taken: start.elapsed(),
    });
    
    if level == CheckLevel::Quick {
        return results;
    }
    
    // Level 3: Moderate checks
    if level >= CheckLevel::Moderate {
        let start = Instant::now();
        if !fermat_test(p, 5, 100_000) {
            results.push(CheckResult {
                passed: false,
                message: "Failed moderate Fermat tests".to_string(),
                time_taken: start.elapsed(),
            });
            return results;
        }
        results.push(CheckResult {
            passed: true,
            message: "Passed moderate Fermat tests".to_string(),
            time_taken: start.elapsed(),
        });
    }
    
    if level == CheckLevel::Moderate {
        return results;
    }
    
    // Level 4: Thorough checks
    if level >= CheckLevel::Thorough {
        let start = Instant::now();
        if !fermat_test(p, 10, 1_000_000) {
            results.push(CheckResult {
                passed: false,
                message: "Failed thorough Fermat tests".to_string(),
                time_taken: start.elapsed(),
            });
            return results;
        }
        results.push(CheckResult {
            passed: true,
            message: "Passed thorough Fermat tests".to_string(),
            time_taken: start.elapsed(),
        });
    }
    
    if level == CheckLevel::Thorough {
        return results;
    }
    
    // Level 5: Exhaustive checks
    if level == CheckLevel::Exhaustive {
        let start = Instant::now();
        if !fermat_test(p, 20, 10_000_000) {
            results.push(CheckResult {
                passed: false,
                message: "Failed exhaustive Fermat tests".to_string(),
                time_taken: start.elapsed(),
            });
            return results;
        }
        results.push(CheckResult {
            passed: true,
            message: "Passed exhaustive Fermat tests".to_string(),
            time_taken: start.elapsed(),
        });
    }
    
    results
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
        assert!(fermat_test(31, 5, 1000));
        // M32 is known to be composite
        assert!(!fermat_test(32, 5, 1000));
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