#!/bin/bash

cargo build --release
objcopy -O ihex target/thumbv6m-none-eabi/release/stm32f072-usb-test file.hex
dfu-tool convert dfuse file.hex file.dfu
dfu-util -a 0 -D file.dfu

