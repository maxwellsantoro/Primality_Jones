#!/usr/bin/env python3
"""
Example usage of primality_jones Python bindings.
"""

import primality_jones as pj
from datetime import datetime

def format_time(ns):
    """Format nanoseconds into a human-readable string."""
    if ns < 1000:
        return f"{ns}ns"
    elif ns < 1_000_000:
        return f"{ns/1000:.2f}µs"
    elif ns < 1_000_000_000:
        return f"{ns/1_000_000:.2f}ms"
    else:
        return f"{ns/1_000_000_000:.2f}s"

def check_mersenne_number(p, level=None):
    """Check a Mersenne number with optional level selection."""
    if level is None:
        level = pj.PyCheckLevel.TrialFactoring
    
    print(f"\nAnalyzing M{p} (2^{p} - 1):")
    print(f"Using check level: {level.description()}")
    print(f"Started at: {datetime.now().strftime('%H:%M:%S')}")
    
    results = pj.check_mersenne(p, level)
    
    print("\nResults:")
    all_passed = True
    for i, result in enumerate(results, 1):
        passed = "✓" if result["passed"] else "✗"
        time_str = format_time(result["time_taken_ns"])
        print(f"{passed} Check {i}: {result['message']} (took {time_str})")
        all_passed &= result["passed"]
    
    if all_passed:
        print(f"\n✓ M{p} remains a promising candidate")
    else:
        print(f"\n✗ M{p} can be eliminated")
    
    print(f"Completed at: {datetime.now().strftime('%H:%M:%S')}")
    return all_passed

def main():
    # Example 1: Check a known Mersenne prime (M31)
    print("Example 1: Testing M31 (known Mersenne prime)")
    check_mersenne_number(31)
    
    # Example 2: Check a known composite (M32)
    print("\nExample 2: Testing M32 (known composite)")
    check_mersenne_number(32)
    
    # Example 3: Using different check levels
    print("\nExample 3: Testing M127 with different levels")
    for level in [pj.PyCheckLevel.PreScreen, pj.PyCheckLevel.TrialFactoring, pj.PyCheckLevel.Probabilistic]:
        check_mersenne_number(127, level)
    
    # Example 4: Using individual test functions
    print("\nExample 4: Using individual test functions")
    p = 11213
    print(f"Testing M{p}:")
    print(f"Is {p} prime? {pj.is_prime_py(p)}")
    if factor := pj.find_small_factors(p, 1_000_000):
        print(f"Found small factor: {factor}")
    print(f"Lucas-Lehmer test: {'passed' if pj.lucas_lehmer(p) else 'failed'}")

if __name__ == "__main__":
    main() 