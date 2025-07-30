/*!
Primality Jones: A Mersenne Number Primality Testing Library

This library provides tools for efficiently testing the primality of Mersenne numbers
(numbers of the form 2^p - 1). It implements various levels of testing, from quick
checks to thorough verification.

# Example

```rust
use primality_jones::{CheckLevel, check_mersenne_candidate};

let p = 12301; // Test M12301
let results = check_mersenne_candidate(p, CheckLevel::LucasLehmer);

if results.iter().all(|r| r.passed) {
    println!("M{} is prime!", p);
} else {
    println!("M{} is not prime.", p);
}
```

# Safety and Performance

This library is designed for mathematical research and should not be used for
cryptographic purposes. The probabilistic nature of Fermat tests means false
positives are possible.

For large Mersenne numbers (>100M digits), consider using the GIMPS software
for definitive primality testing.
*/

use indicatif::{ProgressBar, ProgressStyle};
use num_bigint::{BigUint, RandBigInt};
use num_traits::{One, Zero};
use pyo3::prelude::*;
use pyo3::types::PyDict;
use rand::thread_rng;
use std::time::{Duration, Instant};

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
    /// Pre-screen: Check if the exponent p itself is prime
    PreScreen,
    /// Trial factoring: Check for small factors using special properties
    TrialFactoring,
    /// Probabilistic: Miller-Rabin test (replaces Fermat test)
    Probabilistic,
    /// Lucas-Lehmer: The definitive test for Mersenne primes
    LucasLehmer,
}

impl CheckLevel {
    /// Get a human-readable description of the check level
    pub fn description(&self) -> String {
        match self {
            CheckLevel::PreScreen => "Pre-screen: Check if exponent is prime (instant)".to_string(),
            CheckLevel::TrialFactoring => {
                "Trial factoring: Check for small factors (~1 second)".to_string()
            }
            CheckLevel::Probabilistic => {
                "Probabilistic: Miller-Rabin test (seconds to minutes)".to_string()
            }
            CheckLevel::LucasLehmer => {
                "Lucas-Lehmer: Definitive test (minutes to hours)".to_string()
            }
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
    if n <= 1 {
        return false;
    }
    if n <= 3 {
        return true;
    }
    if n % 2 == 0 || n % 3 == 0 {
        return false;
    }

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

/// Optimized modulo operation for Mersenne numbers M_p = 2^p - 1
///
/// This function implements the bitwise trick for computing k mod (2^p - 1):
/// repeatedly take the bits beyond position p, shift them down, and add them
/// to the lower p bits until no bits remain above p.
///
/// This is much faster than general-purpose modulo for Mersenne numbers.
///
/// # Arguments
///
/// * `k` - The number to reduce modulo M_p
/// * `p` - The Mersenne exponent (M_p = 2^p - 1)
///
/// # Returns
///
/// * k mod (2^p - 1)
///
/// # Algorithm
///
/// For a number k and M_p = 2^p - 1:
/// 1. While k has bits beyond position p:
///    a. Take the high bits (bits p and above)
///    b. Shift them down by p positions
///    c. Add them to the low bits (bits 0 to p-1)
/// 2. The result is k mod M_p
///
/// This works because 2^p ≡ 1 (mod M_p), so shifting by p positions
/// is equivalent to multiplying by 2^p ≡ 1.
pub fn mod_mp(k: &BigUint, p: u64) -> BigUint {
    let mp = (BigUint::one() << p) - BigUint::one();
    
    // Handle edge cases
    if k.is_zero() {
        return BigUint::zero();
    }
    if k == &mp {
        return BigUint::zero();
    }
    if k < &mp {
        return k.clone();
    }
    
    let mut result = k.clone();
    let mut iterations = 0;
    let max_iterations = 1000; // Safety limit
    
    // Keep reducing until result < M_p
    while result > mp && iterations < max_iterations {
        // Split result into high and low parts
        let high_bits = &result >> p;
        let low_bits = &result & &mp;
        
        // Add high bits to low bits
        result = high_bits + low_bits;
        iterations += 1;
    }
    
    // If we hit the iteration limit, fall back to standard modulo
    if iterations >= max_iterations {
        return k % &mp;
    }
    
    // CRITICAL FIX: If the final result is exactly mp, it should be 0
    if result == mp {
        BigUint::zero()
    } else {
        result
    }
}

/// Optimized square and subtract 2 modulo M_p for Lucas-Lehmer test
///
/// This function computes (s^2 - 2) mod M_p using the optimized modulo
/// operation, which is much faster than general-purpose arithmetic.
///
/// # Arguments
///
/// * `s` - The current value in the Lucas-Lehmer sequence
/// * `p` - The Mersenne exponent (M_p = 2^p - 1)
///
/// # Returns
///
/// * (s^2 - 2) mod M_p
pub fn square_and_subtract_two_mod_mp(s: &BigUint, p: u64) -> BigUint {
    let squared = s * s;
    let minus_two = squared - BigUint::from(2u32);
    mod_mp(&minus_two, p)
}

/// Perform a Miller-Rabin primality test with specified parameters
///
/// The Miller-Rabin test is a probabilistic primality test that is strictly stronger
/// than the Fermat test. It can detect all strong pseudoprimes and is the standard
/// for probabilistic primality testing.
///
/// # Arguments
///
/// * `p` - The Mersenne exponent to test (testing 2^p - 1)
/// * `k` - Number of rounds of testing (higher k = lower probability of false positive)
/// * `start_time` - Start time of the test
/// * `timeout` - Timeout for the test
///
/// # Returns
///
/// * `true` if all tests pass (number is probably prime)
/// * `false` if any test fails (number is definitely composite)
///
/// # Algorithm
///
/// For a Mersenne number M_p = 2^p - 1:
/// 1. Write M_p - 1 = 2^s * d where d is odd
/// 2. For each round, choose a random base a in [2, M_p-1]
/// 3. Compute x = a^d mod M_p
/// 4. If x == 1 or x == M_p-1, continue to next round
/// 5. For r = 1 to s-1, compute x = x^2 mod M_p
/// 6. If x == 1, the number is composite
/// 7. If x == M_p-1, continue to next round
/// 8. If we reach here, the number is composite
/// 9. If all rounds pass, the number is probably prime
pub fn miller_rabin_test(p: u64, k: u32, start_time: Instant, timeout: Duration) -> bool {
    let m = (BigUint::one() << p) - BigUint::one();
    let m_minus_1 = &m - BigUint::one();
    let mut rng = thread_rng();

    // Write m-1 = 2^s * d where d is odd
    let mut s = 0;
    let mut d = m_minus_1.clone();
    while &d % BigUint::from(2u32) == BigUint::zero() {
        s += 1;
        d /= BigUint::from(2u32);
    }

    // Create progress bar for Miller-Rabin tests
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
        let a = rng.gen_biguint_range(&BigUint::from(2u32), &m);

        // Compute x = a^d mod m
        let mut x = a.modpow(&d, &m);

        // If x == 1 or x == m-1, this round passes
        if x == BigUint::one() || x == m_minus_1 {
            pb.inc(1);
            continue;
        }

        // Check x^(2^r) mod m for r = 1 to s-1
        let mut is_witness = true;
        for _r in 1..s {
            x = x.modpow(&BigUint::from(2u32), &m);

            if x == m_minus_1 {
                is_witness = false;
                break;
            }

            if x == BigUint::one() {
                // Found a non-trivial square root of 1, so m is composite
                pb.finish_with_message("Failed");
                return false;
            }
        }

        if is_witness {
            // a is a witness for compositeness
            pb.finish_with_message("Failed");
            return false;
        }

        pb.inc(1);
    }

    pb.finish_with_message("Passed");
    true
}



/// Check a Mersenne number candidate with the specified level of thoroughness
///
/// This is the main entry point for testing Mersenne number candidates. It performs
/// a strict pipeline of tests, failing fast if any test fails.
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
/// let results = check_mersenne_candidate(31, CheckLevel::LucasLehmer);
/// assert!(results.iter().all(|r| r.passed)); // M31 is prime
///
/// let results = check_mersenne_candidate(32, CheckLevel::TrialFactoring);
/// assert!(!results.iter().all(|r| r.passed)); // M32 is composite
/// ```
pub fn check_mersenne_candidate(p: u64, level: CheckLevel) -> Vec<CheckResult> {
    let mut results = Vec::new();
    let start_time = Instant::now();

    // PreScreen: Check if the exponent p itself is prime
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

    if !prime_passed || level == CheckLevel::PreScreen {
        return results;
    }

    // TrialFactoring: Check for small factors
    let check_start = Instant::now();
    if let Some(factor) = check_small_factors(p, 1_000_000) {
        results.push(CheckResult {
            passed: false,
            message: format!("Found small factor: {factor}"),
            time_taken: check_start.elapsed(),
        });
        return results;
    }
    results.push(CheckResult {
        passed: true,
        message: "No small factors found up to 1M".to_string(),
        time_taken: check_start.elapsed(),
    });

    if level == CheckLevel::TrialFactoring {
        return results;
    }

    // Probabilistic: Miller-Rabin test
    let check_start = Instant::now();
    let timeout = Duration::from_secs(300); // 5 minutes
    let miller_rabin_passed = miller_rabin_test(p, 5, start_time, timeout);
    results.push(CheckResult {
        passed: miller_rabin_passed,
        message: if miller_rabin_passed {
            "Passed Miller-Rabin test".to_string()
        } else {
            "Failed Miller-Rabin test".to_string()
        },
        time_taken: check_start.elapsed(),
    });

    if !miller_rabin_passed || level == CheckLevel::Probabilistic {
        return results;
    }

    // LucasLehmer: The definitive test
    let check_start = Instant::now();
    let ll_passed = lucas_lehmer_test(p);
    results.push(CheckResult {
        passed: ll_passed,
        message: if ll_passed {
            "Passed Lucas-Lehmer test (definitive)".to_string()
        } else {
            "Failed Lucas-Lehmer test (definitive)".to_string()
        },
        time_taken: check_start.elapsed(),
    });

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
    while 2 * k * p < limit {
        let q = 2 * k * p + 1;
        if (q % 8 == 1 || q % 8 == 7) && is_prime(q) {
            // Check if q divides 2^p - 1 using modular arithmetic
            // We need to check if 2^p ≡ 1 (mod q)
            let remainder = BigUint::from(2u32).modpow(&BigUint::from(p), &BigUint::from(q));
                            if remainder == BigUint::one() {
                    // Don't count M_p itself as a factor
                    let m_p = (BigUint::one() << p) - BigUint::one();
                    if BigUint::from(q) != m_p {
                        return Some(q);
                    }
                }
        }
        k += 1;
    }
    None
}

/// Perform the Lucas-Lehmer test for Mersenne number primality
///
/// This is the definitive test for Mersenne primes. For a Mersenne number M_p = 2^p - 1:
/// 1. Start with s = 4
/// 2. For p-2 iterations, compute s = (s^2 - 2) mod M_p
/// 3. M_p is prime if and only if the final result is s = 0
///
/// # Arguments
///
/// * `p` - The Mersenne exponent to test (testing 2^p - 1)
///
/// # Returns
///
/// * `true` if M_p is prime
/// * `false` if M_p is composite
///
/// # Examples
///
/// ```
/// use primality_jones::lucas_lehmer_test;
///
/// assert!(lucas_lehmer_test(7));   // M7 = 127 is prime
/// assert!(!lucas_lehmer_test(11)); // M11 = 2047 is composite
/// ```
pub fn lucas_lehmer_test(p: u64) -> bool {
    if p < 2 {
        return false;
    }

    let mut s = BigUint::from(4u32);

    // Perform p-2 iterations of the Lucas-Lehmer sequence
    for _ in 0..(p - 2) {
        s = square_and_subtract_two_mod_mp(&s, p);
    }

    // M_p is prime if and only if s = 0
    s == BigUint::zero()
}

#[cfg(test)]
mod tests {
    use super::*;



    #[test]
    fn test_is_prime() {
        assert!(is_prime(31));
        assert!(is_prime(13));
        assert!(!is_prime(15));
        assert!(!is_prime(1));
        assert!(!is_prime(0));
    }

    #[test]
    fn test_miller_rabin_test() {
        // M31 is a known Mersenne prime
        assert!(miller_rabin_test(
            31,
            5,
            Instant::now(),
            Duration::from_secs(30)
        ));
        // M32 is known to be composite
        assert!(!miller_rabin_test(
            32,
            5,
            Instant::now(),
            Duration::from_secs(30)
        ));
    }

    #[test]
    fn test_check_mersenne_candidate() {
        // Test with M7 (known Mersenne prime)
        let results = check_mersenne_candidate(7, CheckLevel::LucasLehmer);
        assert!(results.iter().all(|r| r.passed));

        // Test with M8 (known composite)
        let results = check_mersenne_candidate(8, CheckLevel::TrialFactoring);
        assert!(!results.iter().all(|r| r.passed));
    }

    #[test]
    fn test_large_numbers() {
        // Test handling of a moderately large number
        let results = check_mersenne_candidate(12301, CheckLevel::PreScreen);
        // Should at least complete without panicking
        assert!(results.len() > 0);
    }

    #[test]
    fn test_lucas_lehmer() {
        // Test known Mersenne primes
        assert!(lucas_lehmer_test(7)); // M7 = 127 is prime
        assert!(lucas_lehmer_test(13)); // M13 = 8191 is prime
        assert!(lucas_lehmer_test(17)); // M17 = 131071 is prime
        assert!(lucas_lehmer_test(19)); // M19 = 524287 is prime
        assert!(lucas_lehmer_test(31)); // M31 = 2147483647 is prime

        // Test known composite Mersenne numbers
        assert!(!lucas_lehmer_test(11)); // M11 = 2047 = 23 * 89
        assert!(!lucas_lehmer_test(23)); // M23 = 8388607 = 47 * 178481
        assert!(!lucas_lehmer_test(29)); // M29 = 536870911 = 233 * 1103 * 2089
    }

    #[test]
    fn test_mod_mp() {
        // Test basic cases
        let p = 7;
        let mp = (BigUint::one() << p) - BigUint::one(); // M7 = 127

        // Test that mod_mp gives the same result as regular modulo
        let test_cases = vec![
            BigUint::from(100u32),
            BigUint::from(200u32),
            BigUint::from(500u32),
            BigUint::from(1000u32),
        ];

        for k in test_cases {
            let expected = &k % &mp;
            let actual = mod_mp(&k, p);
            assert_eq!(
                actual, expected,
                "mod_mp({}, {}) = {}, expected {}",
                k, p, actual, expected
            );
        }

        // Test edge cases
        assert_eq!(mod_mp(&mp, p), BigUint::zero()); // M_p mod M_p = 0
        assert_eq!(mod_mp(&BigUint::zero(), p), BigUint::zero()); // 0 mod M_p = 0
        assert_eq!(mod_mp(&BigUint::one(), p), BigUint::one()); // 1 mod M_p = 1
        
        // Test the critical edge case: when reduction results in exactly M_p
        let test_value = &mp + &BigUint::from(100u32); // M_p + 100
        let reduced = mod_mp(&test_value, p);
        assert!(reduced < mp, "Reduced value should be less than M_p");
        assert_eq!(mod_mp(&reduced, p), reduced, "Reduced value should be stable");
    }
}

/// Python module for Mersenne number primality testing
#[pymodule]
fn primality_jones(_py: Python, m: &PyModule) -> PyResult<()> {
    // Expose CheckLevel enum to Python
    #[pyclass]
    #[derive(Clone, Copy)]
    enum PyCheckLevel {
        PreScreen = 0,
        TrialFactoring = 1,
        Probabilistic = 2,
        LucasLehmer = 3,
    }

    #[pymethods]
    impl PyCheckLevel {
        fn description(&self) -> String {
            match self {
                PyCheckLevel::PreScreen => {
                    "Pre-screen: Check if exponent is prime (instant)".to_string()
                }
                PyCheckLevel::TrialFactoring => {
                    "Trial factoring: Check for small factors (~1 second)".to_string()
                }
                PyCheckLevel::Probabilistic => {
                    "Probabilistic: Miller-Rabin test (seconds to minutes)".to_string()
                }
                PyCheckLevel::LucasLehmer => {
                    "Lucas-Lehmer: Definitive test (minutes to hours)".to_string()
                }
            }
        }
    }

    /// Check a Mersenne number for primality
    #[pyfunction]
    fn check_mersenne(p: u64, level: PyCheckLevel) -> PyResult<Vec<PyObject>> {
        let check_level = match level {
            PyCheckLevel::PreScreen => CheckLevel::PreScreen,
            PyCheckLevel::TrialFactoring => CheckLevel::TrialFactoring,
            PyCheckLevel::Probabilistic => CheckLevel::Probabilistic,
            PyCheckLevel::LucasLehmer => CheckLevel::LucasLehmer,
        };

        let results = check_mersenne_candidate(p, check_level);

        Python::with_gil(|py| {
            results
                .into_iter()
                .map(|r| {
                    let dict = PyDict::new(py);
                    dict.set_item("passed", r.passed)?;
                    dict.set_item("message", r.message)?;
                    dict.set_item("time_taken_ns", r.time_taken.as_nanos())?;
                    Ok(dict.into())
                })
                .collect()
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

    /// Perform Lucas-Lehmer test
    #[pyfunction]
    fn lucas_lehmer(p: u64) -> bool {
        lucas_lehmer_test(p)
    }

    // Register Python functions and classes
    m.add_class::<PyCheckLevel>()?;
    m.add_function(wrap_pyfunction!(check_mersenne, m)?)?;
    m.add_function(wrap_pyfunction!(is_prime_py, m)?)?;
    m.add_function(wrap_pyfunction!(find_small_factors, m)?)?;
    m.add_function(wrap_pyfunction!(lucas_lehmer, m)?)?;

    Ok(())
}
