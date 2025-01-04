# rp235x-sha256
Hardware abstraction for the RP2350 (Raspberry Pi Pico 2) SHA256 feature

## Building example

```
RUSTFLAGS="-C link-arg=--nmagic -C link-arg=-Tmemory.x" cargo build --target=riscv32imac-unknown-none-elf --example cryptodongle`
#picotool load -u -v -x -t elf
```

## Running example

```
echo -n "abc" >> /dev/ttyACM0
```

```
stty -F /dev/ttyACM0 raw -echo -echoe -echok && cat /dev/ttyACM0
```
