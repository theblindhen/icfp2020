use crate::value_tree::*;
use num_bigint::BigInt;
use num_traits::{Zero, Num};

pub fn vnil() -> ValueTree {
    ValueTree::VNil
}

pub fn vi64(i: i64) -> ValueTree { vint(i.into()) }

pub fn vint(i: BigInt) -> ValueTree {
    ValueTree::VInt(i)
}

pub fn vcons(hd: ValueTree, tl: ValueTree) -> ValueTree {
    ValueTree::VCons(Box::from((hd, tl)))
}

fn modulate_int(val: &BigInt) -> String {
    let mut encoding = String::from("");
    let mut val = val.clone();

    // encode sign
    if (val < Zero::zero()) {
        encoding.push_str("10");
        val = -val;
    } else {
        encoding.push_str("01");
    }

    // encode width (unary representation)
    let width = int_width(&val);
    let width_encoding = "1".repeat((width / 4) as usize) + "0";
    encoding.push_str(&width_encoding);

    // encode number (binary representation)
    if (val > Zero::zero()) {
        let val_encoding = &format!("{:b}", val);
        let padding = width - (val_encoding.len() as i32);
        let padded_val_encoding = "0".repeat(padding as usize) + val_encoding;
        encoding.push_str(&padded_val_encoding);
    }

    return encoding;
}

fn demodulate_int(s: &str) -> (BigInt, &str) {
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
                let tmp = BigInt::from_str_radix(&remainder[0..width], 2).unwrap();
                return (sign * tmp, &remainder[width..]);
            } else {
                return (Zero::zero(), &remainder[1..]);
            }
        }
        _ => panic!("invalid encoding of integer, cannot demodulate: {:?}", s),
    }
}

pub fn demodulate(s: &str) -> (ValueTree, &str) {
    match &s[0..2] {
        // nil
        "00" => (vnil(), &s[2..]),

        // cons
        "11" => {
            let (head, remainder1) = demodulate(&s[2..]);
            let (tail, remainder2) = demodulate(remainder1);
            (vcons(head, tail), remainder2)
        }

        // neg
        "10" | "01" => {
            let (i, remainder) = demodulate_int(s);
            (vint(i), remainder)
        }

        _ => panic!("cannot demodulate: {:?}", s),
    }
}

pub fn modulate(tree: &ValueTree) -> String {
    use ValueTree::*;

    match tree {
        VInt(val) => modulate_int(val),
        VNil => String::from("00"),
        VCons(args) => match args.as_ref() {
            (head, tail) => String::from("11") + &modulate(head) + &modulate(tail),
        },
        _ => panic!("cannot modulate list"),
    }
}

fn int_width(val: &BigInt) -> i32 {
    assert!(val >= &Zero::zero());

    let mut width = 0;
    let mut remaining = val.clone();
    while (remaining > Zero::zero()) {
        width = width + 4;
        remaining = remaining >> 4;
    }

    return width;
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::aplang::*;

    fn i64_width(val: i64) -> i32 { int_width(&val.into()) }
    fn modulate_i64(val: i64) -> String { modulate_int(&val.into()) }

    #[test]
    fn test_int_width() {
        assert_eq!(i64_width(0), 0);
        assert_eq!(i64_width(1), 4);
        assert_eq!(i64_width(7), 4);
        assert_eq!(i64_width(255), 8);
        assert_eq!(i64_width(4095), 12);
    }

    #[test]
    fn test_modulate_int() {
        assert_eq!(modulate_i64(0), "010");
        assert_eq!(modulate_i64(1), "01100001");
        assert_eq!(modulate_i64(-1), "10100001");
        assert_eq!(modulate_i64(2), "01100010");
        assert_eq!(modulate_i64(-2), "10100010");

        assert_eq!(modulate_i64(16), "0111000010000");
        assert_eq!(modulate_i64(-16), "1011000010000");

        assert_eq!(modulate_i64(255), "0111011111111");
        assert_eq!(modulate_i64(-255), "1011011111111");
        assert_eq!(modulate_i64(256), "011110000100000000");
        assert_eq!(modulate_i64(-256), "101110000100000000");
    }

    #[test]
    fn test_demodulate_int() {
        assert_eq!(demodulate_int("010"), (0.into(), ""));
        assert_eq!(demodulate_int("01100001"), (1.into(), ""));
        assert_eq!(demodulate_int("10100001"), ((-1).into(), ""));
        assert_eq!(demodulate_int("01100010"), (2.into(), ""));
        assert_eq!(demodulate_int("10100010"), ((-2).into(), ""));

        assert_eq!(demodulate_int("0111000010000"), (16.into(), ""));
        assert_eq!(demodulate_int("1011000010000"), ((-16).into(), ""));

        assert_eq!(demodulate_int("0111011111111"), (255.into(), ""));
        assert_eq!(demodulate_int("1011011111111"), ((-255).into(), ""));
        assert_eq!(demodulate_int("011110000100000000"), (256.into(), ""));
        assert_eq!(demodulate_int("101110000100000000"), ((-256).into(), ""));
    }

    #[test]
    fn test_modulate() {
        use ValueTree::*;

        assert_eq!(modulate(&vnil()), "00");
        assert_eq!(modulate(&vcons(vnil(), vnil())), "110000");
        assert_eq!(modulate(&vcons(vi64(0), vnil())), "1101000");
        assert_eq!(modulate(&vcons(vi64(1), vi64(2))), "110110000101100010");
        assert_eq!(
            modulate(&vcons(vi64(1), vcons(vi64(2), vnil()))),
            "1101100001110110001000"
        );
        let inner_list = vcons(vi64(2), vcons(vi64(3), vnil()));
        assert_eq!(
            modulate(&vcons(vi64(1), vcons(inner_list, vcons(vi64(4), vnil())))),
            "1101100001111101100010110110001100110110010000"
        );
    }
    #[test]
    fn test_demodulate() {
        assert_eq!(demodulate("00"), (vnil(), ""));
        assert_eq!(demodulate("110000"), (vcons(vnil(), vnil()), ""));
        assert_eq!(demodulate("1101000"), (vcons(vi64(0), vnil()), ""));
        assert_eq!(demodulate("110110000101100010"), (vcons(vi64(1), vi64(2)), ""));
        assert_eq!(
            demodulate("1101100001110110001000"),
            (vcons(vi64(1), vcons(vi64(2), vnil())), "")
        );
        let inner_list = vcons(vi64(2), vcons(vi64(3), vnil()));
        assert_eq!(
            demodulate("1101100001111101100010110110001100110110010000"),
            (vcons(vi64(1), vcons(inner_list, vcons(vi64(4), vnil()))), "")
        );
    }
}
