# About
This is a tiny rust kernel proof of concept project, written entirely in rust (plus one Assembly file)
The code is extremely ugly, so don't look at it for too long.

## Disclaimer
This code was paritally written by and in support of AI, especially Copilot.
I am not trying to hide it, but I also study full time plus work part time, which means I need to manage my time somehow.
Nevertheless, this does not invalidate my knowledge in this project, since every line of code has been reviewed by me personally, and I still wrote a lot of it myself, even if it might have been edited and improved by Copilot.
I reject having a project fully "vibe coded" by AI and not knowing how anything works at all.

# Pre-Requesits
- limine (bootloader)
- qemu (emulator)
- cargo (rust package manager)
- rustup (rust version and toolchain manager)

## Installation
### Arch
```
yay -S limine qemu rustup
```

### Others
Too lazy to specify for other OSs

# Setup

## Build from Source
1. Clone the repo.
  ```
  git clone https://github.com/Havelex/rost.git
  ```
2. Build the project
  ```
  make
  ```
3. Done.

## From Release
1. Download the .iso from the Release.
2. Done

# Run
1. Run the .iso in qemu by issuing the command in the directory where the .iso lies:
```
qemu-system-x86_64 \
  -cdrom rost.iso \
  -serial file:serial.log \
  -monitor stdio \
  -d int \
  -D qemu.log \
  -no-reboot \
  -no-shutdown \
  -machine hpet=off \
  -M accel=tcg
```
2. 
<img width="941" height="1157" alt="image" src="https://github.com/user-attachments/assets/e1dd0a91-c61c-470d-8420-1ce0091ef3dc" />





