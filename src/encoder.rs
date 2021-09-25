use super::common::*;

pub struct BoxEncoder {
    state: RansState,
}

impl BoxEncoder {
    pub fn new() -> BoxEncoder {
        BoxEncoder { state: RANS64_L }
    }

    pub fn put_symbol(&mut self, output: &mut Vec<u32>, symbol: &EncoderSymbol, scale_bits: u32) {
        assert!(symbol.freq != 0);

        // renormalise
        let mut x = self.state;
        let x_max = ((RANS64_L >> scale_bits) << 32) * symbol.freq as u64;
        if x >= x_max {
            output.push(x as u32);
            x >>= 32;
        }

        let q = mul_hi(x, symbol.rcp_freq) >> symbol.rcp_shift;
        self.state = x + symbol.bias as u64 + q * symbol.cmpl_freq as u64;
    }

    pub fn flush(&mut self, output: &mut Vec<u32>) {
        let x = self.state;

        output.push((x >> 32) as u32);
        output.push((x >> 0) as u32);
    }
}

pub struct EncoderSymbol {
    pub rcp_freq: u64,  // Fixed-point reciprocal frequency
    pub freq: u32,      // Symbol frequency
    pub bias: u32,      // Bias
    pub cmpl_freq: u32, // Complement of frequency: (1 << scale_bits) - freq
    pub rcp_shift: u32, // Reciprocal shift
}

impl EncoderSymbol {
    pub fn new(start: u32, freq: u32, scale_bits: u32) -> EncoderSymbol {
        assert!(scale_bits <= 31);
        assert!(start <= (1 << scale_bits));
        assert!(freq <= (1 << scale_bits) - start);

        if freq < 2 {
            EncoderSymbol {
                rcp_freq: !0,
                rcp_shift: 0,
                bias: start + (1 << scale_bits) - 1,
                freq,
                cmpl_freq: (1 << scale_bits) - freq,
            }
        } else {
            let mut shift = 0u32;
            let (mut x0, x1, t0, t1): (u64, u64, u64, u64);
            while freq > (1 << shift) {
                shift += 1;
            }

            x0 = freq as u64 - 1;
            x1 = 1 << (shift + 31);

            t1 = x1 / freq as u64;
            x0 += (x1 % freq as u64) << 32;
            t0 = x0 / freq as u64;

            EncoderSymbol {
                rcp_freq: t0 + (t1 << 32),
                rcp_shift: shift - 1,
                bias: start,
                freq,
                cmpl_freq: (1 << scale_bits) - freq,
            }
        }
    }
}
