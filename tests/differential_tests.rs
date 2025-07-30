use primality_jones::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GimpsTestResult {
    exponent: u64,
    is_prime: bool,
    discovered_by: Option<String>,
    discovery_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DifferentialTestSuite {
    known_mersenne_primes: Vec<u64>,
    known_composite_mersenne: Vec<u64>,
    test_results: HashMap<u64, GimpsTestResult>,
}

impl DifferentialTestSuite {
    fn new() -> Self {
        // Load test data from external JSON files
        let known_mersenne_primes = Self::load_mersenne_primes()
            .unwrap_or_else(|_| vec![2, 3, 5, 7, 13, 17, 19, 31, 61, 89, 107, 127]);
        
        let known_composite_mersenne = Self::load_composite_mersenne()
            .unwrap_or_else(|_| vec![11, 23, 29, 37, 41, 43, 47, 53, 59, 67, 71, 73, 79, 83, 97]);
        
        Self {
            known_mersenne_primes,
            known_composite_mersenne,
            test_results: HashMap::new(),
        }
    }

    fn load_mersenne_primes() -> Result<Vec<u64>, Box<dyn std::error::Error>> {
        let content = fs::read_to_string("test_data/known_mersenne_primes.json")?;
        Ok(serde_json::from_str(&content)?)
    }

    fn load_composite_mersenne() -> Result<Vec<u64>, Box<dyn std::error::Error>> {
        let content = fs::read_to_string("test_data/known_composite_mersenne.json")?;
        Ok(serde_json::from_str(&content)?)
    }

    fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        Ok(serde_json::from_str(&content)?)
    }

    fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    fn run_differential_tests(&self) -> DifferentialTestReport {
        let mut report = DifferentialTestReport::new();
        
        // Test only smaller known Mersenne primes (up to M127)
        let small_mersenne_primes: Vec<u64> = self.known_mersenne_primes
            .iter()
            .filter(|&&p| p <= 127)
            .cloned()
            .collect();
        
        for &p in &small_mersenne_primes {
            let result = self.test_single_exponent(p, true);
            report.add_result(result);
        }
        
        // Test only smaller known composite Mersenne numbers (up to 127)
        let small_composite_mersenne: Vec<u64> = self.known_composite_mersenne
            .iter()
            .filter(|&&p| p <= 127)
            .cloned()
            .collect();
        
        for &p in &small_composite_mersenne {
            let result = self.test_single_exponent(p, false);
            report.add_result(result);
        }
        
        report
    }

    fn test_single_exponent(&self, p: u64, expected_prime: bool) -> SingleTestResult {
        let start_time = std::time::Instant::now();
        
        // Run Lucas-Lehmer test (the definitive test)
        let ll_result = lucas_lehmer_test(p);
        let ll_time = start_time.elapsed();
        
        // Run Miller-Rabin test for comparison (with shorter timeout for large numbers)
        let mr_start = std::time::Instant::now();
        let timeout = if p > 100 {
            std::time::Duration::from_secs(5) // Shorter timeout for large numbers
        } else {
            std::time::Duration::from_secs(30)
        };
        let mr_result = miller_rabin_test(p, 5, mr_start, timeout);
        let mr_time = mr_start.elapsed();
        
        SingleTestResult {
            exponent: p,
            expected_prime,
            lucas_lehmer_result: ll_result,
            miller_rabin_result: mr_result,
            lucas_lehmer_time: ll_time,
            miller_rabin_time: mr_time,
            lucas_lehmer_correct: ll_result == expected_prime,
            miller_rabin_correct: mr_result == expected_prime,
        }
    }
}

#[derive(Debug, Clone)]
struct SingleTestResult {
    exponent: u64,
    expected_prime: bool,
    lucas_lehmer_result: bool,
    miller_rabin_result: bool,
    lucas_lehmer_time: std::time::Duration,
    miller_rabin_time: std::time::Duration,
    lucas_lehmer_correct: bool,
    miller_rabin_correct: bool,
}

#[derive(Debug)]
struct DifferentialTestReport {
    total_tests: usize,
    lucas_lehmer_correct: usize,
    miller_rabin_correct: usize,
    lucas_lehmer_false_positives: usize,
    lucas_lehmer_false_negatives: usize,
    miller_rabin_false_positives: usize,
    miller_rabin_false_negatives: usize,
    results: Vec<SingleTestResult>,
}

impl DifferentialTestReport {
    fn new() -> Self {
        Self {
            total_tests: 0,
            lucas_lehmer_correct: 0,
            miller_rabin_correct: 0,
            lucas_lehmer_false_positives: 0,
            lucas_lehmer_false_negatives: 0,
            miller_rabin_false_positives: 0,
            miller_rabin_false_negatives: 0,
            results: Vec::new(),
        }
    }

    fn add_result(&mut self, result: SingleTestResult) {
        self.total_tests += 1;
        
        if result.lucas_lehmer_correct {
            self.lucas_lehmer_correct += 1;
        } else if result.expected_prime && !result.lucas_lehmer_result {
            self.lucas_lehmer_false_negatives += 1;
        } else if !result.expected_prime && result.lucas_lehmer_result {
            self.lucas_lehmer_false_positives += 1;
        }
        
        if result.miller_rabin_correct {
            self.miller_rabin_correct += 1;
        } else if result.expected_prime && !result.miller_rabin_result {
            self.miller_rabin_false_negatives += 1;
        } else if !result.expected_prime && result.miller_rabin_result {
            self.miller_rabin_false_positives += 1;
        }
        
        self.results.push(result);
    }

    fn print_summary(&self) {
        println!("=== Differential Test Report ===");
        println!("Total tests: {}", self.total_tests);
        println!();
        
        println!("Lucas-Lehmer Test:");
        println!("  Correct: {}/{} ({:.2}%)", 
            self.lucas_lehmer_correct, self.total_tests, 
            (self.lucas_lehmer_correct as f64 / self.total_tests as f64) * 100.0);
        println!("  False positives: {}", self.lucas_lehmer_false_positives);
        println!("  False negatives: {}", self.lucas_lehmer_false_negatives);
        println!();
        
        println!("Miller-Rabin Test:");
        println!("  Correct: {}/{} ({:.2}%)", 
            self.miller_rabin_correct, self.total_tests, 
            (self.miller_rabin_correct as f64 / self.total_tests as f64) * 100.0);
        println!("  False positives: {}", self.miller_rabin_false_positives);
        println!("  False negatives: {}", self.miller_rabin_false_negatives);
        println!();
        
        if self.lucas_lehmer_false_positives > 0 || self.lucas_lehmer_false_negatives > 0 {
            println!("⚠️  WARNING: Lucas-Lehmer test has errors!");
            for result in &self.results {
                if !result.lucas_lehmer_correct {
                    println!("  M{}: expected {}, got {}", 
                        result.exponent, result.expected_prime, result.lucas_lehmer_result);
                }
            }
        } else {
            println!("✅ Lucas-Lehmer test: PERFECT MATCH with GIMPS data!");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_differential_test_suite() {
        let suite = DifferentialTestSuite::new();
        let report = suite.run_differential_tests();
        
        // The Lucas-Lehmer test should be 100% accurate against known data
        assert_eq!(report.lucas_lehmer_false_positives, 0, 
            "Lucas-Lehmer test produced false positives");
        assert_eq!(report.lucas_lehmer_false_negatives, 0, 
            "Lucas-Lehmer test produced false negatives");
        
        // Print the report
        report.print_summary();
    }

    #[test]
    fn test_known_mersenne_primes() {
        let known_primes = [2, 3, 5, 7, 13, 17, 19, 31, 61, 89, 107, 127];
        
        for &p in &known_primes {
            assert!(lucas_lehmer_test(p), 
                "Known Mersenne prime M{} failed Lucas-Lehmer test", p);
        }
    }

    #[test]
    fn test_known_composite_mersenne() {
        let known_composites = [11, 23, 29, 37, 41, 43, 47, 53, 59, 67, 71, 73, 79, 83, 97];
        
        for &p in &known_composites {
            assert!(!lucas_lehmer_test(p), 
                "Known composite Mersenne number M{} passed Lucas-Lehmer test", p);
        }
    }
} 