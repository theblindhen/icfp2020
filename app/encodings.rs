use crate::aplang::*;

fn modulate_int(mut val: i32) -> String {
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

fn int_width(val: i32) -> i32 {
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

    fn ap(arg1: ApTree, arg2: ApTree) -> ApTree {
        return ApTree::Ap(Box::from((arg1, arg2)));
    }

    fn nil() -> ApTree {
        return ApTree::T(Token::Nil);
    }

    fn cons(head: ApTree, tail: ApTree) -> ApTree {
        return ap(ap(ApTree::T(Token::Cons), head), tail);
    }

    fn int(val: i32) -> ApTree {
        return ApTree::T(Token::Int(val));
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
}
