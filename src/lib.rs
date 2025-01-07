//! Hardware abstraction for the RP235x's (Raspberry Pi Pico 2) built-in SHA256 implementation
#![no_std]

use core::marker::PhantomData;

pub struct Disabled;
pub struct Enabled;
pub struct Running;
//pub struct RunningDma<C: ChannelIndex> {
//    channel: C,
//}

use rp235x_hal::pac;

pub struct Sha256<State> {
    csr: pac::SHA256,
    _state: PhantomData<State>,
}

pub struct Hasher<'a> {
    sha256: &'a mut Sha256<Enabled>,
    /// Internal word cache
    cache: [u8; 4],
    /// Message length in bytes
    count: usize,
}

impl Sha256<Disabled> {
    /// Initialise a SHA256 hasher
    pub fn new(csr: pac::SHA256, resets: &mut pac::RESETS) -> Sha256<Enabled> {
        resets.reset().modify(|_, w| w.sha256().clear_bit());
        while resets.reset_done().read().sha256().bit_is_clear() {}

        // turn off hardware endianess swap
        csr.csr()
            .modify(|_, w| w.start().set_bit().bswap().clear_bit());

        Sha256 {
            csr,
            //            cache: [0; 4],
            //            count: 0,
            _state: PhantomData,
        }
    }
}

impl Sha256<Enabled> {
    pub fn start(&mut self) -> Hasher<'_> {
        Hasher {
            sha256: self,
            cache: [0; 4],
            count: 0,
        }
    }

    //   pub fn start_dma<C: ChannelIndex>(self, channel: Channel<C>) -> Sha256<RunningDma<C>> {
    //       todo!()
    //   }

    /// Compute SHA256 hash of `data`
    pub fn digest(self, input: &[u8]) -> [u8; 32] {
        todo!()
    }
}

impl<'a> Hasher<'a> {
    pub fn write_u8(&mut self, b: u8) {
        let idx = self.count % 4;
        self.cache[idx] = b;
        self.count += 1;

        if idx == 3 {
            let word = u32::from_be_bytes(self.cache);
            self.write_word(word);
            self.cache = [0; 4];
        }
    }

    /// Update hasher state
    pub fn update(&mut self, input: &[u8]) {
        for b in input.iter() {
            self.write_u8(*b);
        }
    }

    fn write_word(&mut self, word: u32) {
        while self.sha256.csr.csr().read().wdata_rdy().bit_is_clear() {
            core::hint::spin_loop();
        }
        self.sha256.csr.wdata().write(|w| unsafe { w.bits(word) });
    }

    pub fn finalize(mut self) -> [u32; 8] {
        // if idx != 0 then there are remaining bytes in the cache
        let idx = self.count % 4;

        // FIPS 180-4 padding
        if idx > 0 {
            self.cache[idx] = 0x80;
            self.write_word(u32::from_be_bytes(self.cache));
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

        let csr = &self.sha256.csr;
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

impl<'a> Drop for Hasher<'a> {
    fn drop(&mut self) {
        self.sha256
            .csr
            .csr()
            .modify(|_, w| w.start().set_bit().bswap().clear_bit());
    }
}
