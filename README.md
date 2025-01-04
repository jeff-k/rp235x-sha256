# rp235x-sha256
Hardware abstraction for the RP2350 (Raspberry Pi Pico 2) SHA256 feature

## Crypto dongle example

### Building
This requires a `memory.x` map for the target RP2350 core.

```
RUSTFLAGS="-C link-arg=--nmagic -C link-arg=-Tmemory.x" cargo build --target=riscv32imac-unknown-none-elf --example cryptodongle
#picotool load -u -v -x -t elf
```

### Running
Listen for serial communication from device:

```
stty -F /dev/ttyACM0 raw -echo -echoe -echok && cat /dev/ttyACM0
```

Send a message to the device to be hashed:

```
echo -n "abc" >> /dev/ttyACM0
```

