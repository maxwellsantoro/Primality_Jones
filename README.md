# Primality Jones

A high-performance Mersenne number primality testing library, written in Rust with Python bindings.

## Features

- Multiple levels of primality testing:
  - **PreScreen**: Check if the exponent p itself is prime (instant)
  - **TrialFactoring**: Check for small factors using special properties (~1 second)
  - **Probabilistic**: Miller-Rabin test (seconds to minutes)
  - **LucasLehmer**: The definitive test for Mersenne primes (minutes to hours)

- Efficient implementations of:
  - Small factor testing using Mersenne number properties
  - Lucas-Lehmer test (the definitive test for Mersenne primes)
  - Optimized modular arithmetic using num-bigint
  - Fermat primality testing with progress reporting

- Available as both:
  - A Rust library
  - A Python module via PyO3 bindings
  - A command-line tool

## Installation

### As a Python Package

```bash
# Create and activate a virtual environment (recommended)
python -m venv .venv
source .venv/bin/activate  # or .venv/bin/activate.fish for fish shell

# Install from source
pip install maturin
git clone https://github.com/maxwellsantoro/primality_jones
cd primality_jones
maturin develop --release
```

### As a Rust Library

Add this to your `Cargo.toml`:

```toml
[dependencies]
primality_jones = { git = "https://github.com/maxwellsantoro/primality_jones" }
```

## Usage

### Python API

```python
import primality_jones as pj

# Basic usage
results = pj.check_mersenne(31, pj.PyCheckLevel.LucasLehmer)
for result in results:
    print(f"{'Passed' if result['passed'] else 'Failed'}: {result['message']}")

# Individual functions
if pj.is_prime_py(31):
    print("Exponent is prime")
    
if factor := pj.find_small_factors(11, 1_000_000):
    print(f"Found small factor: {factor}")
    
if pj.lucas_lehmer(127):
    print("Passed Lucas-Lehmer test")
```

### Rust API

```rust
use primality_jones::{CheckLevel, check_mersenne_candidate};

fn main() {
    let p = 31; // Test M31
    let results = check_mersenne_candidate(p, CheckLevel::LucasLehmer);
    
    if results.iter().all(|r| r.passed) {
        println!("M{} is prime!", p);
    } else {
        println!("M{} is not prime.", p);
    }
}
```

### Command-line Interface

```bash
cargo run --release
```

This will start an interactive session where you can:
- Test individual Mersenne numbers
- Process numbers from a file
- Choose different testing levels
- See detailed progress and results

## Performance Considerations

- For exponents > 1,000,000:
  - Memory usage scales with digits (~0.125 GB per million digits)
  - FastCheck level remains efficient (~1-5 seconds)
  - Higher levels may take significant time
  - Progress bars and timeouts prevent hanging

## Development

### Requirements

- Rust 1.70.0 or higher
- Python 3.7 or higher (for Python bindings)
- maturin (for building Python package)

### Running Tests

```bash
# Rust tests
cargo test

# Python example
python examples/python_usage.py
```

### Building Documentation

```bash
cargo doc --no-deps --open
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Version History

- 0.2.0
  - Added FastCheck level for efficient preliminary testing
  - Added Python bindings via PyO3
  - Improved progress reporting and timeout handling
  - Added memory usage warnings for large exponents
  - Enhanced documentation and examples

- 0.1.0
  - Initial release
  - Basic and Quick level implementations
  - Command-line interface

## Acknowledgments

- Based on properties of Mersenne numbers and primality testing
- Uses optimizations from GIMPS project research
- Inspired by the Great Internet Mersenne Prime Search 