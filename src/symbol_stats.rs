pub struct SymbolStats {
    pub freqs: [u32; 256],
    pub cum_freqs: [u32; 257],
}

impl SymbolStats {
    pub fn new() -> SymbolStats {
        SymbolStats {
            freqs: [0; 256],
            cum_freqs: [0; 257],
        }
    }

    pub fn count_freqs(&mut self, input: &[u8]) {
        self.freqs = [0; 256];
        for i in 0..input.len() {
            self.freqs[input[i] as usize] += 1;
        }
    }

    fn calc_cum_freqs(&mut self) {
        self.cum_freqs[0] = 0;
        for i in 0..256 {
            self.cum_freqs[i + 1] = self.cum_freqs[i] + self.freqs[i];
        }
    }

    pub fn normalise_freqs(&mut self, target_total: u32) {
        assert!(target_total >= 256);

        self.calc_cum_freqs();
        let cur_total = self.cum_freqs[256];

        for i in 1..257 {
            self.cum_freqs[i] =
                ((target_total as u64 * self.cum_freqs[i] as u64) / cur_total as u64) as u32;
        }

        for i in 0..256i32 {
            if self.freqs[i as usize] != 0
                && self.cum_freqs[i as usize + 1] == self.cum_freqs[i as usize]
            {
                let mut best_freq = !0u32;
                let mut best_steal = -1;
                for j in 0..256i32 {
                    let freq = self.cum_freqs[j as usize + 1] - self.cum_freqs[j as usize];
                    if freq > 1 && freq < best_freq {
                        best_freq = freq;
                        best_steal = j;
                    }
                }
                assert!(best_steal != -1);

                if best_steal < i {
                    for j in best_steal + 1..i + 1 {
                        self.cum_freqs[j as usize] -= 1;
                    }
                } else {
                    assert!(best_steal > i);
                    for j in i + 1..best_steal + 1 {
                        self.cum_freqs[j as usize] += 1;
                    }
                }
            }
        }

        assert!(self.cum_freqs[0] == 0 && self.cum_freqs[256] == target_total);
        for i in 0..256 {
            if self.freqs[i] == 0 {
                assert!(self.cum_freqs[i + 1] == self.cum_freqs[i]);
            } else {
                assert!(self.cum_freqs[i + 1] > self.cum_freqs[i]);
            }

            // calc updated freq
            self.freqs[i] = self.cum_freqs[i + 1] - self.cum_freqs[i];
        }
    }
}
