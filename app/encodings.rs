use crate::aplang::*;

fn modulate_int(mut val: i64) -> String {
    let mut encoding = String::from("");

    // encode sign
    if (val < 0) {
        encoding.push_str("10");
        val = -val;
    } else {
        encoding.push_str("01");
    }

    // encode width (unary representation)
    let width = int_width(val);
    let width_encoding = "1".repeat((width / 4) as usize) + "0";
    encoding.push_str(&width_encoding);

    // encode number (binary representation)
    if (val > 0) {
        let val_encoding = &format!("{:b}", val);
        let padding = width - (val_encoding.len() as i32);
        let padded_val_encoding = "0".repeat(padding as usize) + val_encoding;
        encoding.push_str(&padded_val_encoding);
    }

    return encoding;
}

fn demodulate_int(s: &str) -> (i64, &str) {
    let sign = match &s[0..2] {
        "10" => -1,
        "01" => 1,
        _ => panic!("invalid encoding of integer, cannot demodulate: {:?}", s),
    };

    let mut remainder = &s[2..];

    match remainder.find('0') {
        Some(n) => {
            let width = n * 4;
            if width > 0 {
                remainder = &remainder[n + 1..];
                let tmp = i64::from_str_radix(&remainder[0..width], 2).unwrap();
                return (sign * tmp, &remainder[width..]);
            } else {
                return (0, &remainder[1..]);
            }
        }
        _ => panic!("invalid encoding of integer, cannot demodulate: {:?}", s),
    }
}

fn demodulate(s: &str) -> (ApTree, &str) {
    match &s[0..2] {
        // nil
        "00" => (nil(), &s[2..]),

        // cons
        "11" => {
            let (head, remainder1) = demodulate(&s[2..]);
            let (tail, remainder2) = demodulate(remainder1);
            (cons(head, tail), remainder2)
        }

        // neg
        "10" | "01" => {
            let (i, remainder) = demodulate_int(s);
            (int(i), remainder)
        }

        _ => panic!("cannot demodulate: {:?}", s),
    }
}

fn modulate(tree: &ApTree) -> String {
    match tree {
        ApTree::T(Token::Int(val)) => return modulate_int(*val),
        ApTree::T(Token::Nil) => return String::from("00"),
        ApTree::Ap(ap_arg1) => match ap_arg1.as_ref() {
            (ApTree::Ap(ap_arg2), tail) => match ap_arg2.as_ref() {
                (ApTree::T(Token::Cons), head) => {
                    return String::from("11") + &modulate(head) + &modulate(tail);
                }
                _ => panic!("cannot modulate list"),
            },
            _ => panic!("cannot modulate list"),
        },
        _ => panic!("cannot modulate list"),
    }
}

fn int_width(val: i64) -> i32 {
    assert!(val >= 0);

    let mut width = 0;
    let mut remaining = val;
    while (remaining > 0) {
        width = width + 4;
        remaining = remaining >> 4;
    }

    return width;
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::aplang::*;
    #[test]
    fn test_int_width() {
        assert_eq!(int_width(0), 0);
        assert_eq!(int_width(1), 4);
        assert_eq!(int_width(7), 4);
        assert_eq!(int_width(255), 8);
        assert_eq!(int_width(4095), 12);
    }

    #[test]
    fn test_modulate_int() {
        assert_eq!(modulate_int(0), "010");
        assert_eq!(modulate_int(1), "01100001");
        assert_eq!(modulate_int(-1), "10100001");
        assert_eq!(modulate_int(2), "01100010");
        assert_eq!(modulate_int(-2), "10100010");

        assert_eq!(modulate_int(16), "0111000010000");
        assert_eq!(modulate_int(-16), "1011000010000");

        assert_eq!(modulate_int(255), "0111011111111");
        assert_eq!(modulate_int(-255), "1011011111111");
        assert_eq!(modulate_int(256), "011110000100000000");
        assert_eq!(modulate_int(-256), "101110000100000000");
    }

    #[test]
    fn test_demodulate_int() {
        assert_eq!(demodulate_int("010"), (0, ""));
        assert_eq!(demodulate_int("01100001"), (1, ""));
        assert_eq!(demodulate_int("10100001"), (-1, ""));
        assert_eq!(demodulate_int("01100010"), (2, ""));
        assert_eq!(demodulate_int("10100010"), (-2, ""));

        assert_eq!(demodulate_int("0111000010000"), (16, ""));
        assert_eq!(demodulate_int("1011000010000"), (-16, ""));

        assert_eq!(demodulate_int("0111011111111"), (255, ""));
        assert_eq!(demodulate_int("1011011111111"), (-255, ""));
        assert_eq!(demodulate_int("011110000100000000"), (256, ""));
        assert_eq!(demodulate_int("101110000100000000"), (-256, ""));
    }

    #[test]
    fn test_modulate() {
        assert_eq!(modulate(&nil()), "00");
        assert_eq!(modulate(&cons(nil(), nil())), "110000");
        assert_eq!(modulate(&cons(int(0), nil())), "1101000");
        assert_eq!(modulate(&cons(int(1), int(2))), "110110000101100010");
        assert_eq!(
            modulate(&cons(int(1), cons(int(2), nil()))),
            "1101100001110110001000"
        );
        let inner_list = cons(int(2), cons(int(3), nil()));
        assert_eq!(
            modulate(&cons(int(1), cons(inner_list, cons(int(4), nil())))),
            "1101100001111101100010110110001100110110010000"
        );
    }
    #[test]
    fn test_demodulate() {
        assert_eq!(demodulate("00"), (nil(), ""));
        assert_eq!(demodulate("110000"), (cons(nil(), nil()), ""));
        assert_eq!(demodulate("1101000"), (cons(int(0), nil()), ""));
        assert_eq!(demodulate("110110000101100010"), (cons(int(1), int(2)), ""));
        assert_eq!(
            demodulate("1101100001110110001000"),
            (cons(int(1), cons(int(2), nil())), "")
        );
        let inner_list = cons(int(2), cons(int(3), nil()));
        assert_eq!(
            demodulate("1101100001111101100010110110001100110110010000"),
            (cons(int(1), cons(inner_list, cons(int(4), nil()))), "")
        );
    }
}
