#![no_main]

use libfuzzer_sys::fuzz_target;
use primality_jones::{mod_mp, square_and_subtract_two_mod_mp};
use num_bigint::BigUint;
use num_traits::One;

fuzz_target!(|data: &[u8]| {
    if data.len() >= 16 {
        // Extract two u64 values for testing
        let mut bytes1 = [0u8; 8];
        let mut bytes2 = [0u8; 8];
        bytes1.copy_from_slice(&data[..8]);
        bytes2.copy_from_slice(&data[8..16]);
        
        let k = u64::from_le_bytes(bytes1);
        let p = u64::from_le_bytes(bytes2);
        
        // Test with reasonable bounds
        if p > 0 && p <= 1000 && k <= 1000000 {
            let k_big = BigUint::from(k);
            
            // Test mod_mp - should never panic
            let _result = mod_mp(&k_big, p);
            
            // Test square_and_subtract_two_mod_mp with reasonable input
            if k <= 10000 {
                let _result2 = square_and_subtract_two_mod_mp(&k_big, p);
            }
        }
    }
}); 