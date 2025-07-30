use chrono::Local;
use primality_jones::{check_mersenne_candidate, CheckLevel, process_candidates_parallel};
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;
use std::time::{Duration, Instant};
use indicatif::{ProgressBar, ProgressStyle};







fn main() -> io::Result<()> {
    println!("🔍 Primality Jones - Mersenne Number Primality Tester");
    println!("=====================================================");

    // Check if candidates.txt exists
    if !Path::new("candidates.txt").exists() {
        println!("❌ candidates.txt not found. Creating sample file...");
        create_sample_candidates_file()?;
        println!("✅ Created candidates.txt with sample data");
        println!("   Edit this file to add your own Mersenne exponents to test");
        println!("   Each line should contain one exponent (e.g., 31, 61, 89, 107, 127)");
        return Ok(());
    }

    // Read candidates from file
    let candidates = read_candidates_file()?;
    if candidates.is_empty() {
        println!("❌ No valid candidates found in candidates.txt");
        return Ok(());
    }

    println!("📋 Found {} candidates to test", candidates.len());
    println!("   Candidates: {:?}", candidates);

    // Ask user for check level
    let level = get_check_level()?;
    println!("🔬 Using check level: {}", level.description());

    // Process candidates
    let start_time = Instant::now();
    
    if candidates.len() > 1 {
        // Use parallel processing for multiple candidates
        println!("🚀 Using parallel processing for {} candidates", candidates.len());
        let results = process_candidates_parallel(candidates, level);
        
        // Display results
        display_parallel_results(results, start_time);
    } else {
        // Single candidate processing
        let p = candidates[0];
        println!("🔍 Testing M{}...", p);
        
        let results = check_mersenne_candidate(p, level);
        display_single_result(p, results, start_time);
    }

    Ok(())
}

fn create_sample_candidates_file() -> io::Result<()> {
    let mut file = File::create("candidates.txt")?;
    writeln!(file, "# Sample Mersenne exponents to test")?;
    writeln!(file, "# Each line should contain one exponent")?;
    writeln!(file, "# Lines starting with # are ignored")?;
    writeln!(file, "")?;
    writeln!(file, "31")?;
    writeln!(file, "61")?;
    writeln!(file, "89")?;
    writeln!(file, "107")?;
    writeln!(file, "127")?;
    writeln!(file, "")?;
    writeln!(file, "# Add your own exponents below:")?;
    writeln!(file, "# 521")?;
    writeln!(file, "# 607")?;
    writeln!(file, "# 1279")?;
    Ok(())
}

fn read_candidates_file() -> io::Result<Vec<u64>> {
    let file = File::open("candidates.txt")?;
    let reader = BufReader::new(file);
    let mut candidates = Vec::new();

    for (line_num, line) in reader.lines().enumerate() {
        let line = line?;
        let trimmed = line.trim();
        
        // Skip empty lines and comments
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        
        match trimmed.parse::<u64>() {
            Ok(p) => {
                if p > 0 {
                    candidates.push(p);
                } else {
                    eprintln!("⚠️  Warning: Invalid exponent on line {}: {}", line_num + 1, p);
                }
            }
            Err(_) => {
                eprintln!("⚠️  Warning: Could not parse line {}: '{}'", line_num + 1, trimmed);
            }
        }
    }

    Ok(candidates)
}

fn get_check_level() -> io::Result<CheckLevel> {
    println!("\n🔬 Choose check level:");
    println!("1. PreScreen (instant) - Check if exponent is prime");
    println!("2. TrialFactoring (~1s) - Check for small factors");
    println!("3. Probabilistic (seconds-minutes) - Miller-Rabin test");
    println!("4. LucasLehmer (minutes-hours) - Definitive test");
    print!("Enter choice (1-4) [default: 4]: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    
    match input.trim() {
        "1" => Ok(CheckLevel::PreScreen),
        "2" => Ok(CheckLevel::TrialFactoring),
        "3" => Ok(CheckLevel::Probabilistic),
        "4" | "" => Ok(CheckLevel::LucasLehmer),
        _ => {
            println!("Invalid choice, using LucasLehmer");
            Ok(CheckLevel::LucasLehmer)
        }
    }
}

fn display_single_result(p: u64, results: Vec<primality_jones::CheckResult>, start_time: Instant) {
    println!("\n📊 Results for M{}:", p);
    println!("{}", "=".repeat(50));
    
    let mut all_passed = true;
    for (i, result) in results.iter().enumerate() {
        let status = if result.passed { "✅" } else { "❌" };
        println!("{}. {} {}", i + 1, status, result.message);
        println!("   Time: {:?}", result.time_taken);
        
        if !result.passed {
            all_passed = false;
        }
    }
    
    let total_time = start_time.elapsed();
    println!("\n⏱️  Total time: {:?}", total_time);
    
    if all_passed {
        println!("🎉 M{} is PRIME!", p);
    } else {
        println!("💔 M{} is COMPOSITE", p);
    }
}

fn display_parallel_results(results: Vec<(u64, Vec<primality_jones::CheckResult>)>, start_time: Instant) {
    println!("\n📊 Parallel Processing Results:");
    println!("{}", "=".repeat(60));
    
    let mut primes = Vec::new();
    let mut composites = Vec::new();
    
    for (p, candidate_results) in results {
        let all_passed = candidate_results.iter().all(|r| r.passed);
        let total_time: std::time::Duration = candidate_results.iter()
            .map(|r| r.time_taken)
            .sum();
        
        if all_passed {
            primes.push((p, total_time));
            println!("🎉 M{}: PRIME (took {:?})", p, total_time);
        } else {
            composites.push((p, total_time));
            println!("💔 M{}: COMPOSITE (took {:?})", p, total_time);
        }
    }
    
    let total_time = start_time.elapsed();
    println!("\n📈 Summary:");
    println!("   Total time: {:?}", total_time);
    println!("   Primes found: {} ({:?})", primes.len(), primes.iter().map(|(p, _)| format!("M{}", p)).collect::<Vec<_>>().join(", "));
    println!("   Composites: {} ({:?})", composites.len(), composites.iter().map(|(p, _)| format!("M{}", p)).collect::<Vec<_>>().join(", "));
    
    if !primes.is_empty() {
        println!("\n🏆 Mersenne Primes Found:");
        for (p, time) in primes {
            println!("   M{} (took {:?})", p, time);
        }
    }
}
