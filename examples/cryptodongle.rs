//! This example reads bytes from the serial interface and writes
//! back their sha256 sum
#![no_std]
#![no_main]

use panic_halt as _;

use rp235x_hal as hal;

use rp235x_sha256::Sha256;

use embedded_hal::digital::OutputPin;

use core::fmt::Write;
use heapless::String;

use usb_device::{class_prelude::*, prelude::*};
use usbd_serial::SerialPort;

#[unsafe(link_section = ".start_block")]
#[used]
pub static IMAGE_DEF: hal::block::ImageDef = hal::block::ImageDef::secure_exe();

const XTAL_FREQ_HZ: u32 = 12_000_000u32;

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

    pac.RESETS.reset().modify(|_, w| w.sha256().clear_bit());
    while pac.RESETS.reset_done().read().sha256().bit_is_clear() {}

    let usb_bus = UsbBusAllocator::new(hal::usb::UsbBus::new(
        pac.USB,
        pac.USB_DPRAM,
        clocks.usb_clock,
        true,
        &mut pac.RESETS,
    ));

    let mut serial = SerialPort::new(&usb_bus);

    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x16c0, 0x27dd))
        .strings(&[StringDescriptors::default()
            .manufacturer("github.com/jeff-k")
            .product("Crypto dongle example")
            .serial_number("test")])
        .unwrap()
        .device_class(2)
        .build();

    let sio = hal::Sio::new(pac.SIO);

    let pins = hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let mut led_pin = pins.gpio25.into_push_pull_output();

    led_pin.set_low().unwrap();
    loop {
        if usb_dev.poll(&mut [&mut serial]) {
            let mut buf = [0u8; 64];

            match serial.read(&mut buf) {
                Err(_e) => {}
                Ok(0) => {}
                Ok(count) => {
                    let mut response: String<512> = String::new();

                    write!(&mut response, "[ ").unwrap();
                    for byte in buf.iter().take(count) {
                        write!(&mut response, "{:02x} ", byte).unwrap();
                    }
                    write!(&mut response, "]\t").unwrap();

                    led_pin.set_high().unwrap();

                    let mut sha = Sha256::new();
                    buf.iter().take(count).for_each(|b| {
                        sha.write(*b);
                    });

                    let shasum = sha.finalise();
                    led_pin.set_low().unwrap();

                    for word in shasum.iter() {
                        write!(&mut response, "{:08x}", word).unwrap();
                    }

                    let mut wr = response.as_bytes();
                    while !wr.is_empty() {
                        match serial.write(wr) {
                            Ok(len) => wr = &wr[len..],
                            Err(_) => break,
                        }
                    }
                }
            }
        }
    }
}

#[unsafe(link_section = ".bi_entries")]
#[used]
pub static PICOTOOL_ENTRIES: [hal::binary_info::EntryAddr; 5] = [
    hal::binary_info::rp_cargo_bin_name!(),
    hal::binary_info::rp_cargo_version!(),
    hal::binary_info::rp_program_description!(c"Crypto Dongle"),
    hal::binary_info::rp_cargo_homepage_url!(),
    hal::binary_info::rp_program_build_attribute!(),
];
