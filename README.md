# `primality_jones`

[](https://www.google.com/search?q=https://github.com/maxwellsantoro/primality_jones/actions/workflows/ci.yml)
[](https://www.google.com/search?q=https://crates.io/crates/primality_jones)
[](https://www.google.com/search?q=https://docs.rs/primality_jones)
[](https://www.google.com/search?q=https://github.com/maxwellsantoro/primality_jones/blob/main/LICENSE)

A world-class, rigorously verified, high-performance library for testing Mersenne number primality, written in Rust with Python bindings.

This library provides a mathematically correct and highly optimized engine for prime hunters and researchers. It features a cascading test pipeline that quickly eliminates composite candidates and provides definitive proof of primality using the Lucas-Lehmer test.

-----

## Key Features

  * **🔬 Mathematically Verified:** Undergoes an exhaustive three-level verification process, including property-based testing, differential testing against GIMPS data, and a line-by-line algorithmic audit.
  * **🚀 High-Performance Engine:** Utilizes an optimized bitwise modulo operation for Mersenne numbers, providing a 2-5x speedup over standard arithmetic in the critical Lucas-Lehmer test.
  * **⚡ Parallel Processing:** Built with `rayon` to parallelize computations at multiple levels—from trial factoring and Miller-Rabin rounds to processing entire lists of candidates concurrently.
  * **💻 Multiple Interfaces:** Usable as a Rust library, a Python module, or a standalone command-line tool for maximum flexibility.

-----

## 🚀 Quick Start

### Installation

**Rust Library**

```bash
cargo add primality_jones
```

**Python Module**

```bash
pip install primality-jones
```

### Basic Usage

**Rust**

```rust
use primality_jones::{CheckLevel, check_mersenne_candidate};

// Test M127 (a known Mersenne prime)
let results = check_mersenne_candidate(127, CheckLevel::LucasLehmer);

if results.iter().all(|r| r.passed) {
    println!("M127 is prime!");
}
```

**Python**

```python
import primality_jones as pj

# Test M127
results = pj.check_mersenne(127, pj.PyCheckLevel.LucasLehmer)

if all(r['passed'] for r in results):
    print("M127 is prime!")
```

-----

## 🔬 Correctness & Performance

This library was built with an obsessive focus on correctness and speed. The entire verification process and performance characteristics are documented in detail.

  * **For a complete breakdown of the multi-level testing suite, see [VERIFICATION\_SUMMARY.md](https://www.google.com/search?q=./VERIFICATION_SUMMARY.md).**
  * **For detailed benchmarks and optimization notes, see [PERFORMANCE\_SUMMARY.md](https://www.google.com/search?q=./PERFORMANCE_SUMMARY.md).**

-----

## ⚙️ Command-Line Interface

The library includes a powerful CLI for batch processing candidates.

1.  **Create a `candidates.txt` file:**

    ```text
    # Exponents to test, one per line
    # Comments are ignored
    127
    521
    607
    ```

2.  **Run the tester:**

    ```bash
    cargo run --release
    ```

The tool will automatically use parallel processing to test all candidates from the file.

-----

## 🔧 API Overview

The core of the library is the `check_mersenne_candidate` function, which accepts an exponent `p` and a `CheckLevel`.

**Check Levels (in order of execution):**

  * `PreScreen`: Checks if the exponent `p` itself is prime (instant).
  * `TrialFactoring`: Searches for small factors of $M\_p$ using optimized trial division.
  * `Probabilistic`: Runs the strong Miller-Rabin probabilistic primality test.
  * `LucasLehmer`: Performs the definitive Lucas-Lehmer test.

For a complete API reference, please see the [**documentation on docs.rs**](https://www.google.com/search?q=https://docs.rs/primality_jones).

-----

## 🤝 Contributing

Contributions are welcome\! Please ensure that any changes adhere to the project's high standards for mathematical correctness, performance, and testing.

## 📄 License

This project is licensed under the MIT License. See the [LICENSE](https://www.google.com/search?q=./LICENSE) file for details.