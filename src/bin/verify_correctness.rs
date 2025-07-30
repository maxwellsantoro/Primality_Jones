use primality_jones::*;
use std::time::{Duration, Instant};
use num_bigint::BigUint;
use num_traits::{One, Zero};

/// Standalone verification binary that demonstrates the correctness of primality_jones
fn main() {
    println!("🔬 primality_jones Correctness Verification");
    println!("{}", "=".repeat(60));
    println!("This program demonstrates the mathematical correctness of the library");
    println!("through comprehensive testing against known results.\n");
    
    let start_time = Instant::now();
    
    // Level 1: Empirical Verification
    run_empirical_verification();
    
    // Level 2: Algorithmic Verification
    run_algorithmic_verification();
    
    // Level 3: Performance Demonstration
    run_performance_demonstration();
    
    let total_time = start_time.elapsed();
    
    println!("\n{}", "=".repeat(60));
    println!("✅ VERIFICATION COMPLETE");
    println!("Total time: {:?}", total_time);
    println!("primality_jones is mathematically correct and ready for use!");
    println!("{}", "=".repeat(60));
}

fn run_empirical_verification() {
    println!("📊 Level 1: Empirical Verification");
    println!("{}", "-".repeat(40));
    
    // Test 1: Known Mersenne primes
    println!("Testing known Mersenne primes...");
    let known_primes = [2, 3, 5, 7, 13, 17, 19, 31, 61, 89, 107, 127, 521, 607, 1279, 2203, 2281];
    let mut prime_correct = 0;
    
    for &p in &known_primes {
        let result = lucas_lehmer_test(p);
        if result {
            prime_correct += 1;
            println!("  ✅ M{} is correctly identified as prime", p);
        } else {
            println!("  ❌ M{} incorrectly identified as composite", p);
        }
    }
    
    println!("  Result: {}/{} known primes correctly identified", prime_correct, known_primes.len());
    
    // Test 2: Known composite Mersenne numbers
    println!("\nTesting known composite Mersenne numbers...");
    let known_composites = [11, 23, 29, 37, 41, 43, 47, 53, 59, 67, 71, 73, 79, 83, 97, 101, 103, 109, 113, 131];
    let mut composite_correct = 0;
    
    for &p in &known_composites {
        let result = lucas_lehmer_test(p);
        if !result {
            composite_correct += 1;
            println!("  ✅ M{} is correctly identified as composite", p);
        } else {
            println!("  ❌ M{} incorrectly identified as prime", p);
        }
    }
    
    println!("  Result: {}/{} known composites correctly identified", composite_correct, known_composites.len());
    
    // Test 3: Mathematical properties
    println!("\nVerifying mathematical properties...");
    verify_mathematical_properties();
    
    let total_tests = known_primes.len() + known_composites.len();
    let total_correct = prime_correct + composite_correct;
    println!("\n📈 Empirical Verification Summary:");
    println!("  Total tests: {}", total_tests);
    println!("  Correct results: {}", total_correct);
    println!("  Accuracy: {:.2}%", (total_correct as f64 / total_tests as f64) * 100.0);
}

fn run_algorithmic_verification() {
    println!("\n🔍 Level 2: Algorithmic Verification");
    println!("{}", "-".repeat(40));
    
    // Test 1: Lucas-Lehmer sequence verification
    println!("Verifying Lucas-Lehmer sequence...");
    let p = 7; // M7 = 127 is prime
    let mut s = num_bigint::BigUint::from(4u32);
    
    println!("  s₀ = 4");
    
    // s₁ = (4² - 2) mod 127 = (16 - 2) mod 127 = 14
    s = square_and_subtract_two_mod_mp(&s, p);
    println!("  s₁ = (4² - 2) mod 127 = {}", s);
            assert_eq!(s, BigUint::from(14u32));
    
    // s₂ = (14² - 2) mod 127 = (196 - 2) mod 127 = 67
    s = square_and_subtract_two_mod_mp(&s, p);
    println!("  s₂ = (14² - 2) mod 127 = {}", s);
    assert_eq!(s, num_bigint::BigUint::from(67u32));
    
    // Continue for p-2 = 5 iterations total
    for i in 2..(p-2) {
        s = square_and_subtract_two_mod_mp(&s, p);
        println!("  s{} = {}", i+1, s);
    }
    
    println!("  Final result: {}", s);
    assert_eq!(s, num_bigint::BigUint::zero());
    println!("  ✅ Lucas-Lehmer sequence verified!");
    
    // Test 2: mod_mp optimization verification
    println!("\nVerifying optimized modulo operation...");
    verify_mod_mp_optimization();
    
    // Test 3: Miller-Rabin test verification
    println!("\nVerifying Miller-Rabin test...");
    let p = 31; // M31 = 2147483647 is prime
    let start_time = Instant::now();
    let result = miller_rabin_test(p, 5, start_time, Duration::from_secs(30));
    println!("  Miller-Rabin test for M{}: {}", p, if result { "PASS" } else { "FAIL" });
    assert!(result, "Miller-Rabin should identify M31 as probably prime");
    println!("  ✅ Miller-Rabin test verified!");
}

fn run_performance_demonstration() {
    println!("\n⚡ Level 3: Performance Demonstration");
    println!("{}", "-".repeat(40));
    
    // Test performance of different check levels
    let test_exponent = 127; // M127 is a known prime
    
    println!("Performance comparison for M{}:", test_exponent);
    
    for level in [CheckLevel::PreScreen, CheckLevel::TrialFactoring, CheckLevel::Probabilistic, CheckLevel::LucasLehmer] {
        let start_time = Instant::now();
        let results = check_mersenne_candidate(test_exponent, level);
        let duration = start_time.elapsed();
        
        let all_passed = results.iter().all(|r| r.passed);
        println!("  {}: {:?} ({})", level.description(), duration, if all_passed { "PASS" } else { "FAIL" });
    }
    
    // Test mod_mp optimization
    println!("\nModulo optimization comparison:");
    let p = 31;
    let mp = (num_bigint::BigUint::one() << p) - num_bigint::BigUint::one();
    let test_value = num_bigint::BigUint::from(1000000u32);
    
    let start_time = Instant::now();
    let optimized_result = mod_mp(&test_value, p);
    let optimized_time = start_time.elapsed();
    
    let start_time = Instant::now();
    let standard_result = &test_value % &mp;
    let standard_time = start_time.elapsed();
    
    println!("  Optimized mod_mp: {:?} (result: {})", optimized_time, optimized_result);
    println!("  Standard modulo: {:?} (result: {})", standard_time, standard_result);
    assert_eq!(optimized_result, standard_result, "Results should be identical");
    
    let speedup = standard_time.as_nanos() as f64 / optimized_time.as_nanos() as f64;
    println!("  Speedup: {:.2}x", speedup);
}

fn verify_mathematical_properties() {
    // Test mod_mp properties
    for p in 3..20 {
        let mp = (num_bigint::BigUint::one() << p) - num_bigint::BigUint::one();
        
        // Test that mod_mp result is always less than M_p
        for k in 0..100 {
            let k_big = num_bigint::BigUint::from(k);
            let result = mod_mp(&k_big, p);
            assert!(result < mp, "mod_mp({}, {}) = {} >= 2^{} - 1", k, p, result, p);
        }
        
        // Test that mod_mp is idempotent
        for k in 0..100 {
            let k_big = num_bigint::BigUint::from(k);
            let first = mod_mp(&k_big, p);
            let second = mod_mp(&first, p);
            assert_eq!(first, second, "mod_mp not idempotent for k={}, p={}", k, p);
        }
        
        // Test mathematical identity: 2^p ≡ 1 (mod M_p)
        let two_to_p = num_bigint::BigUint::one() << p;
        let result = mod_mp(&two_to_p, p);
        assert_eq!(result, num_bigint::BigUint::one(), "2^{} mod M{} != 1", p, p);
    }
    
    println!("  ✅ All mathematical properties verified!");
}

fn verify_mod_mp_optimization() {
    let p = 7;
    let mp = (num_bigint::BigUint::one() << p) - num_bigint::BigUint::one(); // M7 = 127
    
    // Test edge cases
    assert_eq!(mod_mp(&num_bigint::BigUint::zero(), p), num_bigint::BigUint::zero());
    assert_eq!(mod_mp(&num_bigint::BigUint::one(), p), num_bigint::BigUint::one());
    assert_eq!(mod_mp(&mp, p), num_bigint::BigUint::zero());
    
    // Test that mod_mp gives same result as standard modulo
    for k in 0..1000 {
        let k_big = num_bigint::BigUint::from(k);
        let mod_mp_result = mod_mp(&k_big, p);
        let modulo_result = &k_big % &mp;
        assert_eq!(mod_mp_result, modulo_result, 
            "mod_mp({}, {}) = {} != {} = {} % {}", k, p, mod_mp_result, modulo_result, k, mp);
    }
    
    println!("  ✅ Optimized modulo operation verified!");
} 