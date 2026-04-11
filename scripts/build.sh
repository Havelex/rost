#!/bin/bash
set -e

cargo build -p x86_64

mkdir -p boot/iso_root/boot
cp target/x86_64-rustos/debug/x86_64 boot/iso_root/boot/kernel.elf
cp boot/limine.conf boot/iso_root/boot/
