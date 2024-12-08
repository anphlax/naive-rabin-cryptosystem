use num_bigint::BigInt;
use std::collections::HashMap;
use num_traits::cast::ToPrimitive;

pub const DEFAULT_SYMBOLS: &str = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz(.,;:!?)[<+-*/=>]@| ";


use log::{info, warn, error};
use num_traits::Zero;

pub fn str2num(s: &str, digitstring: &str) -> Option<BigInt> {
    let base = BigInt::from(digitstring.len());
    let mut num = BigInt::zero();

    info!("Encoding string: '{}'", s);
    info!("Using digitstring: '{}'", digitstring);
    info!("Base: {}", base);

    for (i, c) in s.chars().enumerate() {
        if let Some(pos) = digitstring.find(c) {
            info!(
                "Character '{}' at position {} in digitstring: {}",
                c, i, pos
            );
            let pos_value = BigInt::from(pos);
            num = num * &base + pos_value;
            info!("Current number value: {}", num);
        } else {
            warn!("Invalid character '{}' at position {}", c, i);
            return None;
        }
    }

    info!("Final encoded number: {}", num);
    Some(num)
}


pub fn num2str(n: &BigInt, digitstring: &str) -> String {
    let base = BigInt::from(digitstring.len());
    let mut result = String::new();
    let mut current = n.clone();

    info!("Decoding number: {}", n);
    info!("Using digitstring: '{}'", digitstring);
    info!("Base: {}", base);

    if n.is_zero() {
        info!("Special case: Input number is 0");
        return digitstring.chars().next().unwrap().to_string();
    }

    while current > BigInt::zero() {
        let remainder = (&current % &base).to_usize().unwrap();
        info!(
            "Remainder: {}, Corresponding character: '{}'",
            remainder,
            digitstring.chars().nth(remainder).unwrap()
        );
        result.push(digitstring.chars().nth(remainder).unwrap());
        current /= &base;
        info!("Remaining number: {}", current);
    }

    let decoded_string: String = result.chars().rev().collect();
    info!("Final decoded string: '{}'", decoded_string);
    decoded_string
}


#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use super::*;

    #[test]
    fn num2str_simple() {
        let result = num2str(&BigInt::from_str("5028722558842848375853089736952727210229032068167510534250475").unwrap(), DEFAULT_SYMBOLS);
        let expected_result = "Non scholae, sed vitae discimus.";
        assert_eq!(result, expected_result);
    }

    #[test]
    fn test_str2num_basic() {
        use num_bigint::BigInt;
        use num_traits::Zero;

        let digitstring = DEFAULT_SYMBOLS; // Assuming this constant is defined
        let text = "012";
        let base = digitstring.len(); // Base of the custom symbol set

        // "012" -> positions [0, 1, 2], encoded in positional numeral system
        // Expected: 0 * base^2 + 1 * base^1 + 2 * base^0
        let expected_num = BigInt::zero()
            + BigInt::from(0) * BigInt::from(base).pow(2) // 0 * base^2
            + BigInt::from(1) * BigInt::from(base).pow(1) // 1 * base^1
            + BigInt::from(2) * BigInt::from(base).pow(0); // 2 * base^0

        let result = str2num(text, digitstring);

        assert_eq!(result, Some(expected_num));
    }


    #[test]
    fn test_num2str_basic() {
        let expected_text = "abc";
        let number = str2num(expected_text, DEFAULT_SYMBOLS).unwrap();
        let result = num2str(&number, DEFAULT_SYMBOLS);
        assert_eq!(result, expected_text);
    }

    #[test]
    fn test_str2num_and_num2str_round_trip() {
        let text = "HELLO";
        let encoded = str2num(text, DEFAULT_SYMBOLS).unwrap();
        let decoded = num2str(&encoded, DEFAULT_SYMBOLS);

        assert_eq!(decoded, text, "Round-trip encoding and decoding should match the original text");
    }

    #[test]
    fn test_maximum_value() {
        let digitstring = DEFAULT_SYMBOLS;
        let text = "   "; // assuming 'Z' is the highest valid character in `digitstring`

        let encoded = str2num(text, digitstring).unwrap();
        let decoded = num2str(&encoded, digitstring);

        assert_eq!(decoded, text, "The decoded value of the maximum character sequence should match the original");
    }

    // #[test]
    // fn test_invalid_character() {
    //     let text = "HELLO$"; // '$' is not in `DEFAULT_SYMBOLS`, so should handle this gracefully
    //     let result = str2num(text, DEFAULT_SYMBOLS);
    //     assert!(result.is_err(), "Encoding text with invalid characters should return an error, not panic");
    // }
}