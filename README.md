# rp235x-sha256
Hardware abstraction for the RP2350 (Raspberry Pi Pico 2) SHA256 feature

```rust
// Take the peripherals
let mut pac = hal::pac::Peripherals::take().unwrap();

// Initialise the SHA256 hardware
let mut sha = rp235x_sha256::Sha256::new(pac.SHA256, &mut pac.RESETS);

// Build a `Hasher`
let mut hasher = sha.start();

let data = b"abc";

// Update state of hasher
hasher.update(&data);

// Finalize and return 256-bit hash
let hash: [u32; 8] = hasher.finalize();
```

## Crypto dongle example

### Building
This requires a `memory.x` map for the target RP2350 core (eg [RISC-V core memory map](https://github.com/rp-rs/rp-hal/blob/main/rp235x-hal-examples/rp235x_riscv.x)).

```bash
RUSTFLAGS="-C link-arg=--nmagic -C link-arg=-Tmemory.x" cargo build --target=riscv32imac-unknown-none-elf --example cryptodongle
picotool load -u -v -x -t elf target/riscv32imac-unknown-none-elf/debug/examples/cryptodongle
```

### Running
Listen for serial communication from device:

```bash
stty -F /dev/ttyACM0 raw -echo -echoe -echok && cat /dev/ttyACM0
```

Send a message to the device to be hashed:

```bash
echo -n "abc" >> /dev/ttyACM0
```

