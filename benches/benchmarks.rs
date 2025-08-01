use criterion::{black_box, criterion_group, criterion_main, Criterion};
use primality_jones::*;
use num_bigint::BigUint;
use num_traits::{One, Zero};
use rayon::prelude::*;

fn bench_lucas_lehmer_small(c: &mut Criterion) {
    let mut group = c.benchmark_group("Lucas-Lehmer Small");
    group.sample_size(100); // More samples for statistical significance
    
    // Benchmark small known Mersenne primes
    let small_primes = [2, 3, 5, 7, 13, 17, 19, 31];
    
    for &p in &small_primes {
        group.bench_function(&format!("M{}", p), |b| {
            b.iter(|| lucas_lehmer_test(black_box(p)))
        });
    }
    
    group.finish();
}

fn bench_lucas_lehmer_medium(c: &mut Criterion) {
    let mut group = c.benchmark_group("Lucas-Lehmer Medium");
    group.sample_size(50); // Fewer samples for longer tests
    
    // Benchmark medium-sized known Mersenne primes
    let medium_primes = [61, 89, 107, 127];
    
    for &p in &medium_primes {
        group.bench_function(&format!("M{}", p), |b| {
            b.iter(|| lucas_lehmer_test(black_box(p)))
        });
    }
    
    group.finish();
}

fn bench_lucas_lehmer_large(c: &mut Criterion) {
    let mut group = c.benchmark_group("Lucas-Lehmer Large");
    group.sample_size(10); // Few samples for very long tests
    
    // Benchmark larger known Mersenne primes (these will be slower)
    let large_primes = [521, 607, 1279];
    
    for &p in &large_primes {
        group.bench_function(&format!("M{}", p), |b| {
            b.iter(|| lucas_lehmer_test(black_box(p)))
        });
    }
    
    group.finish();
}

fn bench_mod_mp_optimization(c: &mut Criterion) {
    let mut group = c.benchmark_group("Modulo Optimization");
    group.sample_size(100);
    
    // Test the optimized mod_mp function against standard modulo
    let p = 31;
    let mp = (BigUint::one() << p) - BigUint::one();
    let test_values = vec![
        BigUint::from(1000u32),
        BigUint::from(10000u32),
        BigUint::from(100000u32),
        BigUint::from(1000000u32),
    ];
    
    for (i, k) in test_values.iter().enumerate() {
        group.bench_function(&format!("mod_mp_{}", i), |b| {
            b.iter(|| mod_mp(black_box(k), black_box(p)))
        });
        
        group.bench_function(&format!("standard_mod_{}", i), |b| {
            b.iter(|| black_box(k) % black_box(&mp))
        });
    }
    
    group.finish();
}

fn bench_miller_rabin_vs_lucas_lehmer(c: &mut Criterion) {
    let mut group = c.benchmark_group("Miller-Rabin vs Lucas-Lehmer");
    group.sample_size(50);
    
    let test_exponents = [31, 61, 89, 107, 127];
    
    for &p in &test_exponents {
        group.bench_function(&format!("Miller-Rabin_M{}", p), |b| {
            b.iter(|| {
                let start_time = std::time::Instant::now();
                miller_rabin_test(black_box(p), 5, start_time, std::time::Duration::from_secs(30))
            })
        });
        
        group.bench_function(&format!("Lucas-Lehmer_M{}", p), |b| {
            b.iter(|| lucas_lehmer_test(black_box(p)))
        });
    }
    
    group.finish();
}

fn bench_check_mersenne_candidate_levels(c: &mut Criterion) {
    let mut group = c.benchmark_group("Check Levels");
    group.sample_size(100);
    
    let test_exponent = 127; // M127 is a known prime
    
    group.bench_function("PreScreen", |b| {
        b.iter(|| check_mersenne_candidate(black_box(test_exponent), CheckLevel::PreScreen))
    });
    
    group.bench_function("TrialFactoring", |b| {
        b.iter(|| check_mersenne_candidate(black_box(test_exponent), CheckLevel::TrialFactoring))
    });
    
    group.bench_function("Probabilistic", |b| {
        b.iter(|| check_mersenne_candidate(black_box(test_exponent), CheckLevel::Probabilistic))
    });
    
    group.bench_function("LucasLehmer", |b| {
        b.iter(|| check_mersenne_candidate(black_box(test_exponent), CheckLevel::LucasLehmer))
    });
    
    group.finish();
}

fn bench_property_verification(c: &mut Criterion) {
    let mut group = c.benchmark_group("Property Verification");
    group.sample_size(50);
    
    // Benchmark property-based testing scenarios
group.bench_function("mod_mp_bounds_check", |b| {
        b.iter(|| {
            for p in 3..20 {
                let mp = (BigUint::one() << p) - BigUint::one();
                for k in 0..100 {
                    let k_big = BigUint::from(k as u32);
                    let result = mod_mp(&k_big, p);
                    assert!(result <= mp);
                    // Also verify that result is actually less than mp or is zero
                    assert!(result < mp || result == BigUint::zero());
                }
            }
        })
    });
    
    group.bench_function("mod_mp_idempotent_check", |b| {
        b.iter(|| {
            for p in 3..15 {
                for k in 0..100 {
                    let k_big = BigUint::from(k as u32);
                    let first = mod_mp(&k_big, p);
                    let second = mod_mp(&first, p);
                    assert_eq!(first, second);
                }
            }
        })
    });
    
    group.finish();
}

fn bench_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("Memory Usage");
    group.sample_size(20);
    
    // Test memory efficiency for large numbers
    let large_exponents = [521, 607, 1279];
    
    for &p in &large_exponents {
        group.bench_function(&format!("memory_M{}", p), |b| {
            b.iter(|| {
                // This will allocate large BigUint values
                let mp = (BigUint::one() << p) - BigUint::one();
                let _result = lucas_lehmer_test(p);
                // Force cleanup
                drop(mp);
            })
        });
    }
    
    group.finish();
}

fn bench_correctness_verification(c: &mut Criterion) {
    let mut group = c.benchmark_group("Correctness Verification");
    group.sample_size(100);
    
    // Benchmark the verification of known results
    let known_primes = [2, 3, 5, 7, 13, 17, 19, 31, 61, 89, 107, 127];
    let known_composites = [11, 23, 29, 37, 41, 43, 47, 53, 59, 67, 71, 73, 79, 83, 97];
    
    group.bench_function("verify_known_primes", |b| {
        b.iter(|| {
            for &p in &known_primes {
                assert!(lucas_lehmer_test(p));
            }
        })
    });
    
    group.bench_function("verify_known_composites", |b| {
        b.iter(|| {
            for &p in &known_composites {
                assert!(!lucas_lehmer_test(p));
            }
        })
    });
    
    group.finish();
}

// New benchmarks for performance regression detection
fn bench_performance_regression_detection(c: &mut Criterion) {
    let mut group = c.benchmark_group("Performance Regression Detection");
    group.sample_size(200); // More samples for regression detection
    
    // Critical path benchmarks that must not regress
    let critical_exponents = [31, 127, 521];
    
    for &p in &critical_exponents {
        group.bench_function(&format!("critical_lucas_lehmer_M{}", p), |b| {
            b.iter(|| lucas_lehmer_test(black_box(p)))
        });
    }
    
    // Modulo optimization benchmarks
    let p = 31;
    let test_value = BigUint::from(1000000u32);
    
    group.bench_function("critical_mod_mp", |b| {
        b.iter(|| mod_mp(black_box(&test_value), black_box(p)))
    });
    
    group.finish();
}

fn bench_parallel_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("Parallel Performance");
    group.sample_size(50);
    
    // Test parallel processing of multiple candidates
    let candidates = vec![31, 61, 89, 107, 127, 521, 607];
    
    group.bench_function("sequential_lucas_lehmer", |b| {
        b.iter(|| {
            for &p in &candidates {
                black_box(lucas_lehmer_test(p));
            }
        })
    });
    
    group.bench_function("parallel_lucas_lehmer", |b| {
        b.iter(|| {
            candidates.par_iter().for_each(|&p| {
                black_box(lucas_lehmer_test(p));
            });
        })
    });
    
    group.finish();
}

fn bench_scalability(c: &mut Criterion) {
    let mut group = c.benchmark_group("Scalability");
    group.sample_size(10); // Few samples for long-running tests
    
    // Test how performance scales with exponent size
    let exponents = [127, 521, 607, 1279];
    
    for &p in &exponents {
        group.bench_function(&format!("scalability_M{}", p), |b| {
            b.iter(|| lucas_lehmer_test(black_box(p)))
        });
    }
    
    group.finish();
}

criterion_group!(
    benches,
    bench_lucas_lehmer_small,
    bench_lucas_lehmer_medium,
    bench_lucas_lehmer_large,
    bench_mod_mp_optimization,
    bench_miller_rabin_vs_lucas_lehmer,
    bench_check_mersenne_candidate_levels,
    bench_property_verification,
    bench_memory_usage,
    bench_correctness_verification,
    bench_performance_regression_detection,
    bench_parallel_performance,
    bench_scalability,
);
criterion_main!(benches); 