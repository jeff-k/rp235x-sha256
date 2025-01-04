#![no_std]
#![no_main]

use rp235x_hal::pac;

pub struct Sha256 {
    /// Internal word cache
    cache: [u8; 4],
    /// Message length in bytes
    count: usize,
}

impl Sha256 {
    pub fn new() -> Self {
        let csr = unsafe { &*pac::SHA256::ptr() };

        // turn off hardware endianess swap
        csr.csr()
            .modify(|_, w| w.start().set_bit().bswap().clear_bit());
        Self {
            cache: [0; 4],
            count: 0,
        }
    }

    pub fn write(&mut self, b: u8) {
        let idx = self.count % 4;
        self.cache[idx] = b;
        self.count += 1;

        if idx == 3 {
            let word = u32::from_be_bytes(self.cache);
            self.write_word(word);
            self.cache = [0; 4];
        }
    }

    fn write_word(&mut self, word: u32) {
        let csr = unsafe { &*pac::SHA256::ptr() };
        while csr.csr().read().wdata_rdy().bit_is_clear() {
            core::hint::spin_loop();
        }
        csr.wdata().write(|w| unsafe { w.bits(word) });
    }

    // FIPS 180-4 padding
    pub fn finalise(&mut self) -> [u32; 8] {
        let csr = unsafe { &*pac::SHA256::ptr() };

        // if idx != 0 then there are remaining bytes in the cache
        let idx = self.count % 4;

        if idx > 0 {
            self.cache[idx] = 0x80;
            self.write_word(u32::from_le_bytes(self.cache));
            self.count += 4 - idx;
        } else {
            self.write_word(0x80000000);
            self.count += 4;
        }

        // byte count will be a multiple of 4
        let msg_words = self.count / 4;

        // total words to write are the number written + 2 for bit length,
        // rounded to nearest 16
        let total_words = (msg_words + 2 + 15) & !15;

        // number of zeros to pad with
        let zeros = total_words - (msg_words + 2);

        for _ in 0..zeros {
            self.write_word(0);
        }

        // write the bit count to the last 2 words
        let bc: u64 = (self.count - (4 - idx) * (idx > 0) as usize) as u64 * 8;
        self.write_word((bc >> 32) as u32);
        self.write_word(bc as u32);

        // wait for valid sum
        while csr.csr().read().sum_vld().bit_is_clear() {
            core::hint::spin_loop();
        }

        [
            csr.sum0().read().bits(),
            csr.sum1().read().bits(),
            csr.sum2().read().bits(),
            csr.sum3().read().bits(),
            csr.sum4().read().bits(),
            csr.sum5().read().bits(),
            csr.sum6().read().bits(),
            csr.sum7().read().bits(),
        ]
    }
}
