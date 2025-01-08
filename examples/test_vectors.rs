//! Hashes test vectors with the sha256 hardware and compares against references
//! If all tests pass the LED will blink, LED will be solid if there are any failures
#![no_std]
#![no_main]

use panic_halt as _;

use rp235x_hal as hal;

use rp235x_sha256::{Enabled, Sha256};

use embedded_hal::delay::DelayNs;
use embedded_hal::digital::OutputPin;

#[unsafe(link_section = ".start_block")]
#[used]
pub static IMAGE_DEF: hal::block::ImageDef = hal::block::ImageDef::secure_exe();

const XTAL_FREQ_HZ: u32 = 12_000_000u32;

const TEST1: &[u8] = b"asdfasdfasf";
const TEST1HASH: [u8; 32] = sha2_const::Sha256::new().update(TEST1).finalize();

const TEST2: &[u8] = b"abc";
const TEST2HASH: [u8; 32] = sha2_const::Sha256::new().update(TEST2).finalize();

const TEST3: &[u8] = b"";
const TEST3HASH: [u8; 32] = sha2_const::Sha256::new().update(TEST3).finalize();

const TEST4: &[u8] = b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaasdfg";
const TEST4HASH: [u8; 32] = sha2_const::Sha256::new().update(TEST4).finalize();

const TEST5: &[u8] = b"aaaabaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaasdfg";
const TEST5HASH: [u8; 32] = sha2_const::Sha256::new().update(TEST5).finalize();

fn test_hash(sha: &mut Sha256<Enabled>, msg: &[u8], target: &[u8; 32]) -> bool {
    let mut hasher = sha.start();

    hasher.update(msg);

    let shasum: [u32; 8] = hasher.finalize();

    let mut u8arr = [0u8; 32];

    for (i, &word) in shasum.iter().enumerate() {
        u8arr[i * 4..(i + 1) * 4].copy_from_slice(&word.to_be_bytes());
    }

    if &u8arr != target {
        return false;
    }
    true
}

#[hal::entry]
fn main() -> ! {
    let mut pac = hal::pac::Peripherals::take().unwrap();

    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);

    let clocks = hal::clocks::init_clocks_and_plls(
        XTAL_FREQ_HZ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .unwrap();

    let mut timer = hal::Timer::new_timer0(pac.TIMER0, &mut pac.RESETS, &clocks);

    let mut sha = Sha256::new(pac.SHA256, &mut pac.RESETS);

    let sio = hal::Sio::new(pac.SIO);

    let pins = hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let mut led_pin = pins.gpio25.into_push_pull_output();

    led_pin.set_low().unwrap();

    led_pin.set_high().unwrap();

    if !test_hash(&mut sha, TEST1, &TEST1HASH) {
        loop {
            led_pin.set_high().unwrap();
        }
    }

    if !test_hash(&mut sha, TEST2, &TEST2HASH) {
        loop {
            led_pin.set_high().unwrap();
        }
    }

    if !test_hash(&mut sha, TEST3, &TEST3HASH) {
        loop {
            led_pin.set_high().unwrap();
        }
    }

    if !test_hash(&mut sha, TEST4, &TEST4HASH) {
        loop {
            led_pin.set_high().unwrap();
        }
    }

    // Test that incorrect hashes fail
    if test_hash(&mut sha, TEST5, &TEST4HASH) {
        loop {
            led_pin.set_high().unwrap();
        }
    }

    if test_hash(&mut sha, TEST4, &TEST5HASH) {
        loop {
            led_pin.set_high().unwrap();
        }
    }

    loop {
        led_pin.set_high().unwrap();
        timer.delay_ms(200);
        led_pin.set_low().unwrap();
        timer.delay_ms(200);
    }
}

// rp_cargo_bin_name, rp_cargo_homepage_url are in rp-hal main
#[unsafe(link_section = ".bi_entries")]
#[used]
pub static PICOTOOL_ENTRIES: [hal::binary_info::EntryAddr; 3] = [
    //    hal::binary_info::rp_cargo_bin_name!(),
    hal::binary_info::rp_cargo_version!(),
    hal::binary_info::rp_program_description!(c"sha256 test vector evaluator"),
    //    hal::binary_info::rp_cargo_homepage_url!(),
    hal::binary_info::rp_program_build_attribute!(),
];
