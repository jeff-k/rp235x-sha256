//! Hardware abstraction for the RP235x (Raspberry Pi Pico 2) SHA256 accelerator
//!
//! This driver blocks.
//! Reference: https://pip-assets.raspberrypi.com/categories/1214-rp2350/documents/RP-008373-DS-3-rp2350-datasheet.pdf#section_sha256
#![no_std]
#![warn(clippy::pedantic)]
#![warn(missing_docs)]

use core::borrow::BorrowMut;
use rp235x_pac as pac;

pub struct Sha256 {
    csr: pac::SHA256,
}

/// A SHA256 digest
pub struct Hasher<H: BorrowMut<Sha256>> {
    sha256: H,
    /// Internal word cache
    cache: [u8; 4],
    /// Message length in bytes
    count: u64,
}

impl Sha256 {
    /// Initialise a SHA256 hasher
    pub fn new(csr: pac::SHA256, resets: &mut pac::RESETS) -> Self {
        resets.reset().modify(|_, w| w.sha256().clear_bit());
        while resets.reset_done().read().sha256().bit_is_clear() {}

        Sha256 { csr }
    }

    /// Relase the peripheral
    #[must_use]
    pub fn free(self) -> pac::SHA256 {
        self.csr
            .csr()
            .modify(|_, w| w.start().set_bit().bswap().clear_bit());
        self.csr
    }

    #[inline]
    fn wait_for_ready(&self) {
        while self.csr.csr().read().wdata_rdy().bit_is_clear() {
            core::hint::spin_loop();
        }
    }

    #[inline]
    pub fn wait_for_valid(&self) {
        while self.csr.csr().read().sum_vld().bit_is_clear() {
            core::hint::spin_loop();
        }
    }

    #[inline]
    pub fn reset(&mut self) {
        self.csr.csr().modify(|_, w| {
            w.err_wdata_not_rdy()
                .clear_bit_by_one()
                .bswap()
                .set_bit()
                .start()
                .set_bit()
        });
    }

    pub fn start(&mut self) -> Hasher<&mut Self> {
        Hasher::new(self)
    }

    pub fn start_owned(self) -> Hasher<Self> {
        Hasher::new(self)
    }

    /// SAFETY: WDATA accepts u32s
    #[inline]
    fn write_word(&self, word: u32) {
        self.wait_for_ready();
        self.csr.wdata().write(|w| unsafe { w.bits(word) })
    }

    /// Compute SHA256 hash of `data`
    pub fn digest(&mut self, input: &[u8]) -> [u8; 32] {
        let mut hasher = self.start();
        hasher.update(input);
        hasher.finalize()
    }

    fn read_to(&self, output: &mut [u8; 32]) {
        self.wait_for_valid();
        let dev = &self.csr;

        let sums = [
            dev.sum0().read().bits(),
            dev.sum1().read().bits(),
            dev.sum2().read().bits(),
            dev.sum3().read().bits(),
            dev.sum4().read().bits(),
            dev.sum5().read().bits(),
            dev.sum6().read().bits(),
            dev.sum7().read().bits(),
        ];

        for (dst, sum) in output.chunks_exact_mut(4).zip(sums) {
            dst.copy_from_slice(&sum.to_be_bytes());
        }
    }
}

impl<H: BorrowMut<Sha256>> Hasher<H> {
    fn new(mut sha256: H) -> Self {
        sha256.borrow_mut().reset();
        Self {
            sha256,
            cache: [0; 4],
            count: 0,
        }
    }

    fn write_u8(&mut self, b: u8) {
        let idx = (self.count % 4) as usize;
        self.cache[idx] = b;
        self.count += 1;

        if idx == 3 {
            if self.count % 64 == 0 {
                self.sha256.borrow().wait_for_ready();
            }
            self.sha256
                .borrow()
                .write_word(u32::from_le_bytes(self.cache));
            self.cache = [0; 4];
        }
    }

    /// Update hasher state
    pub fn update(&mut self, input: &[u8]) {
        for b in input {
            self.write_u8(*b);
        }
    }

    /// Implements FIPS 180-4 padding
    fn finish_to(&mut self, output: &mut [u8; 32]) {
        let sha256 = self.sha256.borrow();

        // if idx != 0 then there are remaining bytes in the cache
        let cache_idx = self.count % 4;

        // FIPS 180-4 padding
        self.cache[cache_idx as usize] = 0x80;
        sha256.write_word(u32::from_le_bytes(self.cache));

        // the number of times that `write_word` has been called
        let msg_words = (self.count + 4) / 4;

        // total words to write are the number written + 2 for bit length,
        // rounded to nearest 16
        let total_words = (msg_words + 2 + 15) & !15;

        // number of zeros to pad with
        let zeros = total_words - (msg_words + 2);

        for _ in 0..zeros {
            sha256.write_word(0);
        }

        // write the bit count to the last 2 words
        let bc = (self.count * 8).to_be_bytes();
        sha256.write_word(u32::from_le_bytes([bc[0], bc[1], bc[2], bc[3]]));
        sha256.write_word(u32::from_le_bytes([bc[4], bc[5], bc[6], bc[7]]));

        sha256.read_to(output);
    }

    pub fn finish(mut self, output: &mut [u8; 32]) -> H {
        self.finish_to(output);
        self.sha256
    }
}

impl Hasher<Sha256> {
    #[must_use]
    pub fn finalize(mut self) -> (Sha256, [u8; 32]) {
        let mut output = [0; 32];
        self.finish_to(&mut output);
        (self.sha256, output)
    }
}

impl Hasher<&mut Sha256> {
    #[must_use]
    pub fn finalize(mut self) -> [u8; 32] {
        let mut output = [0; 32];
        self.finish_to(&mut output);
        output
    }
}
