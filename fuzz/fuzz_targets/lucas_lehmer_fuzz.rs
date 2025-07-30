#![no_main]

use libfuzzer_sys::fuzz_target;
use primality_jones::lucas_lehmer_test;

fuzz_target!(|data: &[u8]| {
    // Convert bytes to u64 for exponent testing
    if data.len() >= 8 {
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&data[..8]);
        let exponent = u64::from_le_bytes(bytes);
        
        // Test with reasonable bounds to avoid infinite loops
        if exponent > 0 && exponent <= 10000 {
            // This should never panic
            let _result = lucas_lehmer_test(exponent);
        }
    }
}); 