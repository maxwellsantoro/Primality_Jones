use std::fs::File;
use std::io::{self, BufRead, Write};
use std::path::Path;
use std::time::{Instant, Duration};
use chrono::Local;
use primality_jones::{CheckLevel, check_mersenne_candidate};

fn read_candidates<P: AsRef<Path>>(path: P) -> io::Result<Vec<u64>> {
    match File::open(path) {
        Ok(file) => {
            let reader = io::BufReader::new(file);
            let mut candidates = Vec::new();

            for line in reader.lines() {
                if let Ok(line) = line {
                    if line.starts_with('M') {
                        if let Ok(num) = line[1..].parse::<u64>() {
                            candidates.push(num);
                        }
                    }
                }
            }
            Ok(candidates)
        }
        Err(_) => Ok(Vec::new())
    }
}

fn get_user_input() -> io::Result<u64> {
    loop {
        print!("Enter a Mersenne exponent to test (e.g., 31 or M31 for M31 = 2^31 - 1): ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        let input = input.trim();
        if input.is_empty() {
            println!("Exiting...");
            std::process::exit(0);
        }
        
        // Remove 'M' prefix if present
        let number_str = if input.starts_with('M') || input.starts_with('m') {
            &input[1..]
        } else {
            input
        };
        
        match number_str.parse::<u64>() {
            Ok(n) => return Ok(n),
            Err(_) => println!("Please enter a valid positive number (with or without 'M' prefix).")
        }
    }
}

fn get_check_level() -> io::Result<CheckLevel> {
    println!("\nAvailable check levels:");
    println!("1. {}", CheckLevel::Basic.description());
    println!("2. {}", CheckLevel::FastCheck.description());
    println!("3. {}", CheckLevel::Quick.description());
    println!("4. {}", CheckLevel::Moderate.description());
    println!("5. {}", CheckLevel::Thorough.description());
    println!("6. {}", CheckLevel::Exhaustive.description());
    
    print!("\nSelect check level (1-6), or press Enter to start from level 1: ");
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    
    let input = input.trim();
    if input.is_empty() {
        return Ok(CheckLevel::Basic);
    }
    
    Ok(match input.parse::<u32>() {
        Ok(1) => CheckLevel::Basic,
        Ok(2) => CheckLevel::FastCheck,
        Ok(3) => CheckLevel::Quick,
        Ok(4) => CheckLevel::Moderate,
        Ok(5) => CheckLevel::Thorough,
        Ok(6) => CheckLevel::Exhaustive,
        _ => {
            println!("Invalid input, defaulting to Basic checks");
            CheckLevel::Basic
        }
    })
}

fn get_user_choice() -> io::Result<String> {
    print!("\nPress Enter to try next level, 'r' to retry this level, 'c' to change level, 'n' for new number, or 'q' to quit: ");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_lowercase())
}

fn calculate_timeout(p: u64, level: CheckLevel) -> Duration {
    // Base timeouts in seconds
    let base_timeout = match level {
        CheckLevel::Basic => 1,
        CheckLevel::FastCheck => 5,
        CheckLevel::Quick => 30,
        CheckLevel::Moderate => 300,   // 5 minutes
        CheckLevel::Thorough => 1800,  // 30 minutes
        CheckLevel::Exhaustive => 7200, // 2 hours
    };
    
    // For large numbers, scale the timeout based on the size
    if p > 1_000_000 {
        // Calculate scaling factor based on digits
        let digits = (p as f64 * 0.301029995664) as u64;  // log10(2) ≈ 0.301029995664
        let scale_factor = (digits / 1_000_000) as u64 + 1;  // Scale up for each million digits
        Duration::from_secs(base_timeout * scale_factor)
    } else {
        Duration::from_secs(base_timeout)
    }
}

fn check_candidate(p: u64, level: CheckLevel) -> bool {
    println!("\n{}", "=".repeat(60));
    println!("Analyzing M{} (2^{} - 1):", p, p);
    
    // Add warning for very large numbers
    if p > 1_000_000 {
        let digits = (p as f64 * 0.301029995664) as u64;
        println!("\n⚠️  Warning: This is a very large Mersenne number!");
        println!("   - Approximate decimal digits: {}", digits);
        println!("   - Estimated memory required: ~{} GB", (digits as f64 * 0.125 / 1024.0).ceil());
        println!("   - Estimated time for Quick check: {} hours", 
            (digits as f64 * 0.0001).ceil()); // Rough estimate based on digits
        println!("   - Higher level checks will take significantly longer");
        print!("\nDo you want to continue? [y/N]: ");
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        if let Ok(_) = io::stdin().read_line(&mut input) {
            if !input.trim().eq_ignore_ascii_case("y") {
                println!("Skipping this candidate...");
                return false;
            }
        }
    }
    
    println!("Started at: {}", Local::now().format("%H:%M:%S"));
    println!("Using check level: {:?}", level);
    
    let timeout = calculate_timeout(p, level);
    println!("Timeout set to: {:?}", timeout);
    
    let check_start = Instant::now();
    let results = check_mersenne_candidate(p, level);
    
    // Check if we exceeded timeout
    if check_start.elapsed() > timeout {
        println!("\n⚠️  Check timed out after {:?}", timeout);
        println!("Consider using a lower check level for numbers this large.");
        return false;
    }
    
    println!("\nResults for M{}:", p);
    for (i, result) in results.iter().enumerate() {
        println!("Check {}: {} (took {:?})",
            i + 1,
            result.message,
            result.time_taken
        );
    }
    
    let passed = results.iter().all(|r| r.passed);
    if passed {
        println!("\n✓ M{} remains a promising candidate", p);
    } else {
        println!("\n✗ M{} can be eliminated", p);
    }
    
    println!("Completed at: {}", Local::now().format("%H:%M:%S"));
    passed
}

fn main() -> io::Result<()> {
    let start_time = Instant::now();
    let mut candidates = read_candidates("candidates.txt")?;
    
    if candidates.is_empty() {
        println!("No candidates.txt file found or file is empty.");
        println!("Enter numbers interactively (press Enter with no input to exit).");
        candidates.push(get_user_input()?);
    } else {
        println!("Found {} Mersenne candidates", candidates.len());
    }
    
    let mut current_level = get_check_level()?;
    let mut remaining_candidates = candidates;
    
    'main_loop: while !remaining_candidates.is_empty() {
        let mut passed_candidates = Vec::new();
        let mut i = 0;
        
        while i < remaining_candidates.len() {
            let p = remaining_candidates[i];
            if check_candidate(p, current_level) {
                passed_candidates.push(p);
            }
            
            match get_user_choice()?.as_str() {
                "" => {
                    // Move to next level if available
                    match current_level {
                        CheckLevel::Basic => current_level = CheckLevel::FastCheck,
                        CheckLevel::FastCheck => current_level = CheckLevel::Quick,
                        CheckLevel::Quick => current_level = CheckLevel::Moderate,
                        CheckLevel::Moderate => current_level = CheckLevel::Thorough,
                        CheckLevel::Thorough => current_level = CheckLevel::Exhaustive,
                        CheckLevel::Exhaustive => {
                            println!("\nNo more levels available!");
                            break 'main_loop;
                        }
                    }
                    println!("\nMoving to {:?} level...", current_level);
                    i += 1;
                },
                "r" => {
                    // Retry the same candidate at the same level
                    continue;
                },
                "c" => {
                    // Change level
                    current_level = get_check_level()?;
                },
                "n" => {
                    // Test a new number
                    if let Ok(n) = get_user_input() {
                        remaining_candidates = vec![n];
                        i = 0;
                        passed_candidates.clear();
                    }
                },
                "q" => break 'main_loop,
                _ => {
                    println!("Invalid choice, continuing with next candidate...");
                    i += 1;
                }
            }
        }
        
        // Update remaining candidates to only those that passed
        remaining_candidates = passed_candidates;
        
        if remaining_candidates.is_empty() {
            println!("\nNo candidates remain! Enter a new number or press Enter to exit.");
            match get_user_input() {
                Ok(n) => {
                    remaining_candidates = vec![n];
                    current_level = get_check_level()?;
                },
                Err(_) => break 'main_loop
            }
        } else {
            println!("\n{} candidates remain for next level:", remaining_candidates.len());
            for &p in &remaining_candidates {
                println!("M{}", p);
            }
        }
    }
    
    let duration = start_time.elapsed();
    println!("\nTotal runtime: {:?}", duration);
    Ok(())
}
