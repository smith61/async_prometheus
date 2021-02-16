use std::io::Write;

pub(crate) fn is_valid_ident(s: &str) -> bool {
    let mut allow_nums = false;
    for c in s.chars() {
        match c {
            'a'..='z' | 'A'..='Z' | '_' => {}
            '0'..='9' if allow_nums => {}
            _ => {
                return false;
            }
        }

        allow_nums = true;
    }

    s.len() != 0 && !s.starts_with("__")
}

pub(crate) fn write_escaped_string(writer: &mut impl Write, mut s: &str) -> std::io::Result<()> {
    while s.len() != 0 {
        if let Some(index) = memchr::memchr3(b'\n', b'\\', b'"', s.as_bytes()) {
            if index != 0 {
                writer.write_all(&s[..index].as_bytes())?;
                s = &s[index..];
            }

            let mut next_start_index = 0;
            for c in s.chars() {
                let escaped_str = match c {
                    '\n' => {
                        "\\n"
                    },
                    '\\' => {
                        "\\\\"
                    },
                    '"' => {
                        "\\\""
                    },
                    _ => {
                        break;
                    }
                };

                writer.write_all(escaped_str.as_bytes())?;
                next_start_index += 1;
            }

            s = &s[next_start_index..];

        } else {
            writer.write_all(s.as_bytes())?;
            break;
        }
    }

    Ok(())
}

#[cfg(test)]
mod test {

    use crate::utils::{is_valid_ident, write_escaped_string};

    #[test]
    fn test_is_valid_ident() {
        let valid_idents = [
            "ident_without_nums",
            "ident_with_nums_1",
            "_starting_with_underscore"
        ];

        for valid_ident in &valid_idents {
            println!("Testing valid ident: {}", valid_ident);
            assert!(is_valid_ident(valid_ident));
        }

        let invalid_idents = [
            "ident_with_invalid_chars_\n",
            "9_ident_with_leading_num",
            "__ident_with_double_underscore"
        ];

        for invalid_ident in &invalid_idents {
            println!("Testing invalid ident: {}", invalid_ident);
            assert!(!is_valid_ident(invalid_ident));
        }
    }

    #[test]
    fn test_escaped_string() {
        let strings_to_escape = [
            "String with no escapes",
            "String with new\nline",
            "\n\\\""
        ];

        let expected_strings = [
            "String with no escapes",
            "String with new\\nline",
            "\\n\\\\\\\""
        ];

        for (string_to_escape, expected) in strings_to_escape.iter().zip(expected_strings.iter()) {
            let mut buffer = Vec::new();
            write_escaped_string(&mut buffer, string_to_escape).unwrap();
            let escaped_string = String::from_utf8(buffer).unwrap();
            assert_eq!(escaped_string, *expected);
        }
    }
}
