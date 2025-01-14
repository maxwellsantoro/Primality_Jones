# Primality Jones 🔢

A fast Rust library for checking Mersenne number primality candidates. This tool implements various levels of primality testing, from quick checks to thorough verification, making it ideal for preliminary screening of Mersenne number candidates before investing in full Lucas-Lehmer tests.

## Features

- 🚀 Multiple levels of primality testing
- ⚡ Efficient modular arithmetic for Mersenne numbers
- 📊 Detailed progress reporting and timing
- 🔍 Both command-line interface and library API
- 🧮 Specialized for extremely large Mersenne numbers

## Requirements

- Rust 1.70.0 or higher
- Cargo package manager

## Installation

### As a Command-Line Tool

```bash
# Install from git repository
cargo install --git https://github.com/maxwellsantoro/primality_jones

# Run from anywhere
primality_jones
```

### As a Library Dependency

Add to your `Cargo.toml`:
```toml
[dependencies]
primality_jones = { git = "https://github.com/maxwellsantoro/primality_jones" }
```

## Usage

### Command-Line Interface

1. Create a file named `candidates.txt` with your Mersenne number candidates:
```
M12301
M44497
M110503
```

2. Run the tool:
```bash
primality_jones
```

3. Select a check level:
- Level 1 (Basic): Instant divisibility checks
- Level 2 (Quick): Basic Fermat tests (seconds)
- Level 3 (Moderate): Extended Fermat tests (~1 minute)
- Level 4 (Thorough): Multiple methods (~10 minutes)
- Level 5 (Exhaustive): All available methods (hours)

### Library API

```rust
use primality_jones::{CheckLevel, check_mersenne_candidate};

fn main() {
    // Check M12301
    let p = 12301;
    let results = check_mersenne_candidate(p, CheckLevel::Quick);
    
    // Process results
    for result in &results {
        println!("{}: {} (took {:?})", 
            result.message, 
            result.passed, 
            result.time_taken
        );
    }
    
    // Check if it's a promising candidate
    if results.iter().all(|r| r.passed) {
        println!("M{} is a promising candidate!", p);
    } else {
        println!("M{} is not prime.", p);
    }
}
```

## How It Works

The tool uses a multi-stage approach to efficiently screen Mersenne number candidates:

1. **Basic Properties**
   - Checks if the exponent satisfies known Mersenne prime properties
   - Verifies divisibility rules
   - Checks modular congruences

2. **Primality Testing**
   - Tests if the exponent itself is prime
   - Uses optimized trial division

3. **Fermat Tests**
   - Performs probabilistic primality tests
   - Uses efficient modular arithmetic for Mersenne numbers
   - Increases thoroughness with each level

4. **Optimization Techniques**
   - Custom modular reduction for Mersenne numbers
   - Efficient bit operations
   - Early exit on failure

## Performance

The tool is optimized for handling extremely large Mersenne numbers (over 100M digits). It uses:

- Efficient modular arithmetic specifically for Mersenne numbers
- Bit-level operations where possible
- Progressive testing levels to fail fast
- Memory-efficient algorithms

## Testing

Run the test suite:
```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name
```

The test suite includes:
- Unit tests for each primality check
- Integration tests with known Mersenne primes
- Performance benchmarks
- Edge case handling

## Security Considerations

- This tool is for mathematical research and should not be used for cryptographic purposes
- The probabilistic nature of Fermat tests means false positives are possible
- For cryptographic applications, use established cryptographic libraries

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. Areas for improvement:

- Additional primality tests
- Performance optimizations
- GPU acceleration
- Distributed computing support

### Development Setup

1. Clone the repository:
```bash
git clone https://github.com/maxwellsantoro/primality_jones.git
cd primality_jones
```

2. Install dependencies:
```bash
cargo build
```

3. Run tests:
```bash
cargo test
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Based on research in Mersenne prime testing
- Inspired by GIMPS (Great Internet Mersenne Prime Search)
- Uses techniques from modern number theory

## Version History

- 0.1.0 (2024-02-14)
  - Initial release
  - Basic primality testing functionality
  - Command-line interface
  - Library API 