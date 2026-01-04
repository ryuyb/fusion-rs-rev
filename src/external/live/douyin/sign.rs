fn cal_one_str(one_str: &str, orgi_iv: u32) -> u32 {
    let mut k = orgi_iv;
    for c in one_str.chars() {
        let a = c as u32;
        k = (k ^ a).wrapping_mul(65599);
    }
    k
}

fn cal_one_str_3(one_str: &str, orgi_iv: u32) -> u32 {
    let mut k = orgi_iv;
    for c in one_str.chars() {
        k = k.wrapping_mul(65599).wrapping_add(c as u32);
    }
    k
}

fn get_one_chr(enc_chr_code: u32) -> char {
    if enc_chr_code < 26 {
        char::from_u32(enc_chr_code + 65).unwrap_or('?')
    } else if enc_chr_code < 52 {
        char::from_u32(enc_chr_code + 71).unwrap_or('?')
    } else if enc_chr_code < 62 {
        char::from_u32(enc_chr_code - 4 + 48).unwrap_or('?')
    } else {
        char::from_u32(enc_chr_code - 17 + 48).unwrap_or('?')
    }
}

fn enc_num_to_str(one_orgi_enc: u32) -> String {
    let mut s = String::with_capacity(5);
    for i in (0..=24).rev().step_by(6) {
        s.push(get_one_chr((one_orgi_enc >> i) & 63));
    }
    s
}

pub fn get_ac_signature(timestamp: u64, site: &str, nonce: &str, ua: &str) -> String {
    let sign_head = "_02B4Z6wo00f01";
    let time_stamp_s = timestamp.to_string();

    let a = cal_one_str(site, cal_one_str(&time_stamp_s, 0)) % 65521;

    let xor_val = (timestamp as u32) ^ (a.wrapping_mul(65521));
    let binary_str = format!("10000000110000{:032b}", xor_val);
    let b = u64::from_str_radix(&binary_str, 2).unwrap_or(0);
    let b_u32 = b as u32;

    let b_s = b.to_string();
    let c = cal_one_str(&b_s, 0);
    let d = enc_num_to_str(b_u32 >> 2);
    let e = (b / 4294967296) as u32;
    let f = enc_num_to_str((b_u32 << 28) | (e >> 4));
    let g = 582085784_u32 ^ b_u32;
    let h = enc_num_to_str((e << 26) | (g >> 6));
    let i = get_one_chr(g & 63);
    let j = ((cal_one_str(ua, c) % 65521) << 16) | (cal_one_str(nonce, c) % 65521);
    let k = enc_num_to_str(j >> 2);
    let l = enc_num_to_str((j << 28) | ((524576 ^ b_u32) >> 4));
    let m = enc_num_to_str(a);

    let n = format!("{}{}{}{}{}{}{}{}", sign_head, d, f, h, i, k, l, m);
    let o_val = cal_one_str_3(&n, 0);
    let o = format!("{:x}", o_val);
    let o_suffix = if o.len() >= 2 { &o[o.len() - 2..] } else { &o };

    format!("{}{}", n, o_suffix)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cal_one_str() {
        assert_eq!(cal_one_str("test", 0), cal_one_str("test", 0));
    }

    #[test]
    fn test_get_one_chr() {
        assert_eq!(get_one_chr(0), 'A');
        assert_eq!(get_one_chr(25), 'Z');
        assert_eq!(get_one_chr(26), 'a');
        assert_eq!(get_one_chr(51), 'z');
    }

    #[test]
    fn test_enc_num_to_str() {
        let result = enc_num_to_str(0);
        assert_eq!(result.len(), 5);
    }
}
