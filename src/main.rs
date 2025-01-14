use std::fs::File;
use std::io::{self, BufRead, Write};
use std::path::Path;
use std::time::Instant;
use chrono::Local;
use primality_jones::{CheckLevel, check_mersenne_candidate};

fn read_candidates<P: AsRef<Path>>(path: P) -> io::Result<Vec<u64>> {
    let file = File::open(path)?;
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

fn get_check_level() -> io::Result<CheckLevel> {
    println!("\nAvailable check levels:");
    println!("1. {}", CheckLevel::Basic.description());
    println!("2. {}", CheckLevel::Quick.description());
    println!("3. {}", CheckLevel::Moderate.description());
    println!("4. {}", CheckLevel::Thorough.description());
    println!("5. {}", CheckLevel::Exhaustive.description());
    
    print!("\nSelect check level (1-5), or press Enter to start from level 1: ");
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    
    let input = input.trim();
    if input.is_empty() {
        return Ok(CheckLevel::Basic);
    }
    
    Ok(match input.parse::<u32>() {
        Ok(1) => CheckLevel::Basic,
        Ok(2) => CheckLevel::Quick,
        Ok(3) => CheckLevel::Moderate,
        Ok(4) => CheckLevel::Thorough,
        Ok(5) => CheckLevel::Exhaustive,
        _ => {
            println!("Invalid input, defaulting to Basic checks");
            CheckLevel::Basic
        }
    })
}

fn get_user_choice() -> io::Result<String> {
    print!("\nPress Enter to try next level, 'r' to retry this level, 'c' to change level, or 'q' to quit: ");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_lowercase())
}

fn check_candidate(p: u64, level: CheckLevel) -> bool {
    println!("\n{}", "=".repeat(60));
    println!("Analyzing M{} (2^{} - 1):", p, p);
    println!("Started at: {}", Local::now().format("%H:%M:%S"));
    println!("Using check level: {:?}", level);
    
    let results = check_mersenne_candidate(p, level);
    
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
    let candidates = read_candidates("candidates.txt")?;
    println!("Found {} Mersenne candidates", candidates.len());
    
    let mut current_level = get_check_level()?;
    let mut remaining_candidates: Vec<u64> = candidates;
    
    'main_loop: while !remaining_candidates.is_empty() {
        let mut passed_candidates = Vec::new();
        
        for &p in &remaining_candidates {
            if check_candidate(p, current_level) {
                passed_candidates.push(p);
            }
            
            match get_user_choice()?.as_str() {
                "" => {
                    // Move to next level if available
                    match current_level {
                        CheckLevel::Basic => current_level = CheckLevel::Quick,
                        CheckLevel::Quick => current_level = CheckLevel::Moderate,
                        CheckLevel::Moderate => current_level = CheckLevel::Thorough,
                        CheckLevel::Thorough => current_level = CheckLevel::Exhaustive,
                        CheckLevel::Exhaustive => {
                            println!("\nNo more levels available!");
                            break 'main_loop;
                        }
                    }
                    println!("\nMoving to {:?} level...", current_level);
                },
                "r" => {
                    // Retry the same candidate at the same level
                    continue;
                },
                "c" => {
                    // Change level
                    current_level = get_check_level()?;
                },
                "q" => break 'main_loop,
                _ => {
                    println!("Invalid choice, continuing with next candidate...");
                }
            }
        }
        
        // Update remaining candidates to only those that passed
        remaining_candidates = passed_candidates;
        
        if remaining_candidates.is_empty() {
            println!("\nNo candidates remain!");
            break;
        }
        
        println!("\n{} candidates remain for next level:", remaining_candidates.len());
        for &p in &remaining_candidates {
            println!("M{}", p);
        }
    }
    
    let duration = start_time.elapsed();
    println!("\nTotal runtime: {:?}", duration);
    Ok(())
}
