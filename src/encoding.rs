use num_bigint::BigUint;
use std::collections::HashMap;
use num_traits::cast::ToPrimitive;
use num_bigint::BigInt;

pub const DEFAULT_SYMBOLS: &str = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz(.,;:!?)[<+-*/=>]@| ";

pub fn str2num(text: &str, digitstring: &str) -> Result<BigInt, String> {
    // create hashmao for characters to index mapping with optimized lookup time
    let char_map: HashMap<char, usize> = digitstring.chars()
        .enumerate()
        .map(|(index, char)| {
            (char, index)
        })
        .collect();

    let radix = BigInt::from(digitstring.len());

    let mut result = BigInt::from(0);

    let mut invalid_chars = Vec::new();

    for (index, char) in text.chars().enumerate() {
        if let Some(&char_index) = char_map.get(&char) {
            // for each character, multiply the current result by the radix and add the characters index.
            result = result * &radix + BigInt::from(char_index)
        } else {
            invalid_chars.push((index, char));

            // Return an error if an invalid character is found
            return Err(format!("Invalid character '{}' at position {}", char, index));
        }
    }

    // Logging
    println!("{}", result);

    if !invalid_chars.is_empty() {
        println!("found invalid chars");
    }

    Ok(result)
}

pub fn num2str(number: BigInt, digitstring: &str) -> String {
    // determine the custom radix based on the length of the digit string
    let radix = digitstring.len();

    // create hashmap for characters to index mapping with optimized lookup time
    let char_map: HashMap<usize, char> = digitstring.chars()
        .enumerate()
        .map(|(index, char)| {
            (index, char)
        })
        .collect();

    // Convert the BigInt number to a string in the custom radix
    let mut number_copy = number.clone();
    let mut decoded_number_str = String::new();

    while number_copy > BigInt::from(0) {
        let remainder = &number_copy % radix;
        decoded_number_str.push(*char_map.get(&(remainder.to_usize().unwrap())).unwrap());
        number_copy /= radix;
    }

    // reverse the string since we processed the number from the least significant bit
    decoded_number_str.chars().rev().collect()

}


#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use super::*;

    #[test]
    fn num2str_simple() {
        let result = num2str(BigInt::from_str("5028722558842848375853089736952727210229032068167510534250475").unwrap(), DEFAULT_SYMBOLS);
        let expected_result = "Non scholae, sed vitae discimus.";
        assert_eq!(result, expected_result);
    }

    #[test]
    fn test_str2num_basic() {
        let digitstring = DEFAULT_SYMBOLS;
        let text = "012";
        // Expected to map to "012" -> positions [0, 1, 2]
        // In base `digitstring.len()`, this should produce a BigInt representation
        let expected_num = BigInt::from(0) * BigInt::from(digitstring.len())
            + BigInt::from(1) * BigInt::from(digitstring.len())
            + BigInt::from(2);

        let result = str2num(text, digitstring);
        assert_eq!(result, Ok(expected_num));
    }

    #[test]
    fn test_num2str_basic() {
        let expected_text = "abc";
        let number = str2num(expected_text, DEFAULT_SYMBOLS).unwrap();
        let result = num2str(number, DEFAULT_SYMBOLS);
        assert_eq!(result, expected_text);
    }

    #[test]
    fn test_str2num_and_num2str_round_trip() {
        let text = "HELLO";
        let encoded = str2num(text, DEFAULT_SYMBOLS).unwrap();
        let decoded = num2str(encoded, DEFAULT_SYMBOLS);

        assert_eq!(decoded, text, "Round-trip encoding and decoding should match the original text");
    }

    #[test]
    fn test_maximum_value() {
        let digitstring = DEFAULT_SYMBOLS;
        let text = "   "; // assuming 'Z' is the highest valid character in `digitstring`

        let encoded = str2num(text, digitstring).unwrap();
        let decoded = num2str(encoded, digitstring);

        assert_eq!(decoded, text, "The decoded value of the maximum character sequence should match the original");
    }

    #[test]
    fn test_invalid_character() {
        let text = "HELLO$"; // '$' is not in `DEFAULT_SYMBOLS`, so should handle this gracefully
        let result = str2num(text, DEFAULT_SYMBOLS);
        assert!(result.is_err(), "Encoding text with invalid characters should return an error, not panic");
    }
}