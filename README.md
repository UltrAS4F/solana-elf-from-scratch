# Building a Solana ELF from scratch

## Introduction

In this document, we'll manually build a complete sBPF ELF from scratch, examining every single byte along the way. It is based on the foundational work by [@deanmlittle](https://github.com/deanmlittle) on [sBPF program deconstruction](https://gist.github.com/deanmlittle/d3a8e4c9e4a4929fe3f8cbdba7959859). I highly recommend reading it first to understand the broader context.

If you'd like to follow along, create a new `.so` file and open it in a hex editor, then type the bytes in manually. Otherwise, you can simply open the `abort.so` file and read it.

## Target Program

For this experiment, we'll use a small SBPF program [sbpf-asm-abort](https://github.com/deanmlittle/sbpf-asm-abort):

```assembly
.globl e
e:
  lddw r0, 1
  exit
```

This program:
- Loads the immediate value `1` into register `r0` using the `lddw` instruction
- Exits


## File Layout Overview

Here's how the file is organized:

| Offset Range | Content | Size |
|--------------|---------|------|
| 0x0000–0x003F | ELF Header | 64 bytes |
| 0x0040–0x0057 | `.text` section | 24 bytes |
| 0x0058–0x0068 | `.shstrtab` string table | 17 bytes |
| 0x0069–0x006F | Alignment padding | 7 bytes |
| 0x0070–0x00AF | Section Header [0] (NULL) | 64 bytes |
| 0x00B0–0x00EF | Section Header [1] (`.text`) | 64 bytes |
| 0x00F0–0x012F | Section Header [2] (`.shstrtab`) | 64 bytes |

Total file size: **304 bytes**

## Step 1: ELF Header (0x0000–0x003F)

The ELF header provides essential metadata to understand and process the file.

### ELF Header Structure

Here's the Rust struct that defines the ELF header:

```rust
pub struct ELFHeader {
    pub ei_magic: [u8; 4],
    pub ei_class: u8,
    pub ei_data: u8,
    pub ei_version: u8,
    pub ei_osabi: u8,
    pub ei_abiversion: u8,
    pub ei_pad: [u8; 7],
    pub e_type: u16,
    pub e_machine: u16,
    pub e_version: u32,
    pub e_entry: u64,
    pub e_phoff: u64,
    pub e_shoff: u64,
    pub e_flags: u32,
    pub e_ehsize: u16,
    pub e_phentsize: u16,
    pub e_phnum: u16,
    pub e_shentsize: u16,
    pub e_shnum: u16,
    pub e_shstrndx: u16,
}
```

### Complete 64-byte Header Breakdown

| Offset | Size | Field | Description | Value (Little-Endian) |
|--------|------|-------|-------------|----------------------|
| 0x00 | 4 | ei_magic | ELF file signature | `7F 45 4C 46` ("\x7FELF") |
| 0x04 | 1 | ei_class | 64-bit architecture | `02` |
| 0x05 | 1 | ei_data | Little-endian byte order | `01` |
| 0x06 | 1 | ei_version | ELF version | `01` |
| 0x07 | 1 | ei_osabi | Operating system ABI | `00` |
| 0x08 | 1 | ei_abiversion | ABI version | `00` |
| 0x09 | 7 | ei_pad | Reserved padding | `00 00 00 00 00 00 00` |
| 0x10 | 2 | e_type | File type | `03 00` |
| 0x12 | 2 | e_machine | Target architecture (BPF) | `F7 00` (247) |
| 0x14 | 4 | e_version | Object file version | `01 00 00 00` |
| 0x18 | 8 | e_entry | Entry point address | `40 00 00 00 00 00 00 00` (64) |
| 0x20 | 8 | e_phoff | Program header offset | `00 00 00 00 00 00 00 00` |
| 0x28 | 8 | e_shoff | Section header offset | `70 00 00 00 00 00 00 00` (112) |
| 0x30 | 4 | e_flags | Processor-specific flags | `00 00 00 00` |
| 0x34 | 2 | e_ehsize | ELF header size | `40 00` (64) |
| 0x36 | 2 | e_phentsize | Program header entry size | `38 00` (56) |
| 0x38 | 2 | e_phnum | Number of program headers | `00 00` (0) |
| 0x3A | 2 | e_shentsize | Section header entry size | `40 00` (64) |
| 0x3C | 2 | e_shnum | Number of section headers | `03 00` (3) |
| 0x3E | 2 | e_shstrndx | Section name string table index | `02 00` (2) |

### Key Calculations

**Entry Point (`e_entry = 0x40`)**: Since we have no program headers, the code starts immediately after the ELF header at offset 64 (0x40).

**Section Header Offset (`e_shoff = 0x70`)**:
1. All sections end at offset 0x68
2. Add alignment padding: `(8 - (0x69 % 8)) % 8 = 7` bytes
3. Section headers start at: `0x69 + 7 = 0x70`

## Step 2: Program Headers

The program is static as it contains no dynamically linked symbols (no syscalls or function calls). Due to this, the ELF doesn't require a program headers section* which makes things simpler.

## Step 3: Code Section (0x0040–0x0057)

The `.text` section contains the actual sBPF instructions.

The instruction format per 8-byte word is: `opcode (1 byte) | dst/src (1 byte) | offset (2 bytes) | imm32 (4 bytes)`. The `lddw` instruction uses two 8-byte words.

### Instruction Breakdown

**`lddw r0, 1` (16 bytes at 0x40–0x4F)**:
- `18 00 00 00 01 00 00 00 00 00 00 00 00 00 00 00`

**`exit` (8 bytes at 0x50–0x57)**:
- `95 00 00 00 00 00 00 00`

## Step 4: String Table (0x0058–0x0068)

The `.shstrtab` section contains null-terminated strings for section names:

| Offset | Content | String |
|--------|---------|--------|
| 0x58 | `00` | "" (empty string for NULL section) |
| 0x59 | `2E 74 65 78 74 00` | ".text\0" |
| 0x5F | `2E 73 68 73 74 72 74 61 62 00` | ".shstrtab\0" |

**Size**: 17 bytes total. The section headers reference these strings by their offsets within this table.

## Step 5: Alignment Padding (0x0069–0x006F)

ELF requires section headers to be aligned on 8-byte boundaries. Since the string table ends at 0x68, we need 7 bytes of padding to reach the next 8-byte boundary at 0x70.

## Step 6: Section Headers (0x0070–0x012F)

### Section Header Structure

Here's the Rust struct that defines the section headers:

```rust
pub struct SectionHeader {
    pub sh_name: u32,
    pub sh_type: u32,
    pub sh_flags: u64,
    pub sh_addr: u64,
    pub sh_offset: u64,
    pub sh_size: u64,
    pub sh_link: u32,
    pub sh_info: u32,
    pub sh_addralign: u64,
    pub sh_entsize: u64,
}
```

Each section header is 64 bytes and describes one section. We have three:


### Section Header [0] - NULL (0x0070–0x00AF)
All zeros, as required by the ELF spec.

### Section Header [1] - .text (0x00B0–0x00EF)

| Field | Value | Bytes | Description |
|-------|-------|-------|-------------|
| sh_name | 1 | `01 00 00 00` | Offset of ".text" in string table |
| sh_type | 1 | `01 00 00 00` | 1 (program data) |
| sh_flags | 0x6 | `06 00 00 00 00 00 00 00` | ALLOC \| EXEC (allocated, executable) |
| sh_addr | 0x40 | `40 00 00 00 00 00 00 00` | Virtual address (same as file offset) |
| sh_offset | 0x40 | `40 00 00 00 00 00 00 00` | File offset |
| sh_size | 0x18 | `18 00 00 00 00 00 00 00` | Size in bytes (24) |
| sh_link | 0 | `00 00 00 00` | Link to other section (none) |
| sh_info | 0 | `00 00 00 00` | Additional info (none) |
| sh_addralign | 4 | `04 00 00 00 00 00 00 00` | Alignment requirement |
| sh_entsize | 0 | `00 00 00 00 00 00 00 00` | Entry size (none) |

### Section Header [2] - .shstrtab (0x00F0–0x012F)

| Field | Value | Bytes | Description |
|-------|-------|-------|-------------|
| sh_name | 7 | `07 00 00 00` | Offset of ".shstrtab" in string table |
| sh_type | 3 | `03 00 00 00` | 3 (string table) |
| sh_flags | 0 | `00 00 00 00 00 00 00 00` | No special flags |
| sh_addr | 0 | `00 00 00 00 00 00 00 00` | No virtual address |
| sh_offset | 0x58 | `58 00 00 00 00 00 00 00` | File offset |
| sh_size | 0x11 | `11 00 00 00 00 00 00 00` | Size in bytes (17) |
| sh_link | 0 | `00 00 00 00` | Link to other section (none) |
| sh_info | 0 | `00 00 00 00` | Additional info (none) |
| sh_addralign | 1 | `01 00 00 00 00 00 00 00` | Byte alignment |
| sh_entsize | 0 | `00 00 00 00 00 00 00 00` | Entry size (none) |

## Final ELF file

![abort.so.png](/docs/abort.so.png)

## Testing

If you've been typing along so far, congratulations, you're now officially a compiler. You can verify the manually constructed ELF by running:

```bash
cargo test
```

## Conclusion

We've successfully built a complete sBPF ELF executable from scratch, without relying on any build tools. This 304-byte file contains everything needed to load and execute the sBPF program.

## References

- [Deconstructing fib.so, a minimal sBPF program](https://gist.github.com/deanmlittle/d3a8e4c9e4a4929fe3f8cbdba7959859)
- [sBPF bytcode reference](https://github.com/anza-xyz/sbpf/blob/58236a8ca3c3eeddae8b3c7f45a3246d8ee0fb8e/doc/bytecode.md) 
- [elf manual](https://man7.org/linux/man-pages/man5/elf.5.html)
