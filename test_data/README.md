# Test Data

This directory contains external test data files used by the differential tests.

## Files

- `known_mersenne_primes.json` - List of known Mersenne prime exponents (from GIMPS data)
- `known_composite_mersenne.json` - List of known composite Mersenne number exponents

## Usage

The differential tests automatically load these files at runtime. If the files cannot be loaded, the tests fall back to a smaller set of hardcoded values for testing.

## Maintenance

These files can be updated independently of the code to include new discoveries or corrections to the known Mersenne prime data. The JSON format makes it easy to maintain and version control these lists.

## Data Sources

The data is sourced from the Great Internet Mersenne Prime Search (GIMPS) project and other mathematical databases. The lists include all known Mersenne primes and a comprehensive set of composite Mersenne numbers for testing purposes. 