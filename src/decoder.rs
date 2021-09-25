use super::common::*;

pub struct BoxDecoder {
    state: RansState,
}

impl BoxDecoder {
    pub fn new(input: &mut Vec<u32>) -> BoxDecoder {
        let mut x = (input.pop().unwrap() as u64) << 0;
        x |= (input.pop().unwrap() as u64) << 32;

        BoxDecoder { state: x }
    }

    pub fn get(&self, scale_bits: u32) -> u32 {
        self.state as u32 & ((1u32 << scale_bits) - 1)
    }

    pub fn advance_symbol(
        &mut self,
        input: &mut Vec<u32>,
        symbol: &DecoderSymbol,
        scale_bits: u32,
    ) {
        let mask = (1u64 << scale_bits) - 1;

        let mut x = self.state;
        x = symbol.freq as u64 * (x >> scale_bits) + (x & mask) - symbol.start as u64;

        // renormalise
        if x < RANS64_L {
            x = (x << 32) | input.pop().unwrap() as u64;
            assert!(x >= RANS64_L);
        }

        self.state = x;
    }
}

pub struct DecoderSymbol {
    pub start: u32, // Start of range
    pub freq: u32,  // Symbol frequency
}

impl DecoderSymbol {
    pub fn new(start: u32, freq: u32) -> DecoderSymbol {
        assert!(start <= (1 << 31));
        assert!(freq <= (1 << 31) - start);
        DecoderSymbol { start, freq }
    }
}
