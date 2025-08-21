# WireStorm – CoreTech Message Protocol Proxy

![Rust](https://img.shields.io/badge/Rust-2021-orange?logo=rust)
![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)

> **Two Rust implementations of a CoreTech Message Protocol (CTMP) proxy for the WireStorm challenge.**

Both proxies **forward messages from a single source client to multiple destination clients**, while validating message structure.

---

## Table of Contents

- [Features](#features)
- [Repository Structure](#repository-structure)
- [Getting Started](#getting-started)
- [Protocol Details](#protocol-details)
- [Design Notes](#design-notes)
- [License](#license)
- [Contact](#contact)

---

## ✨ Features

- **Part 1:** Basic CTMP relay
- **Part 2:** Extended CTMP relay with checksum validation
- Multi-threaded, concurrent receivers
- Message validation and safe error handling

---

## 📂 Repository Structure

```text
.
├── wirestorm/      # Part 1 – Basic CTMP relay
│   ├── src/
│   │   ├── main.rs
│   │   └── ctmp.rs
│   ├── tests.py
│   └── Cargo.toml
│
├── wirestorm2/     # Part 2 – Extended CTMP with checksum
│   ├── src/
│   │   ├── main.rs
│   │   └── ctmp.rs
│   ├── tests.py
│   └── Cargo.toml
│
└── README.md
```

Each folder is a **standalone Rust project** with its own tests.

---

## 🚀 Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (2021 edition or later)
- Python 3 (for tests)

### Build

```sh
cargo build --release
```

### Run

```sh
./target/release/wirestorm    # Part 1
./target/release/wirestorm2   # Part 2
```

**Default ports:**
- Source client: `33333` - Allows a single connection
- Destination clients: `44444` - Allows multiple connections

### Test

```sh
python3 tests.py
```
Expected output: `OK`

---

## 📡 Protocol Details

### Part 1 – Basic Protocol Relay (`wirestorm/`)

#### CTMP Header

```text
0                   1                   2                   3
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|   MAGIC 0xCC   |   PADDING    |           LENGTH            |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|   PADDING      |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                       DATA ...                              |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
```

- **MAGIC:** `0xCC`
- **LENGTH:** Payload length (u16, network byte order)
- **PADDING:** Reserved `0x00`
- **DATA:** Message payload

**Features:**
- Parses headers and extracts `DATA`
- Forwards messages from **source → multiple destinations**
- Drops **invalid messages**
- Multi-threaded: supports **multiple concurrent receivers**

---

### Part 2 – Extended Protocol with Checksum (`wirestorm2/`)

Adds **OPTIONS** and **CHECKSUM** fields.

#### CTMP Extended Header

```text
0                   1                   2                   3
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|   MAGIC 0xCC   |  OPTIONS     |           LENGTH            |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|   CHECKSUM     |   PADDING    |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                       DATA ...                              |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
```

- **OPTIONS:**
  - Bit 0: Reserved
  - Bit 1: `1` = Sensitive message, `0` = Normal
  - Bits 2–7: Padding
- **CHECKSUM:** 16-bit one’s complement sum of header + data (with checksum field = `0xCCCC`)
- **Sensitive messages:** Must pass checksum validation; otherwise discarded and logged

**Features:**
- All features from Part 1
- **Checksum validation** for sensitive messages
- Safe discard of invalid sensitive messages

---

## 🛠️ Design Notes

- **Concurrency:** Each receiver runs in a thread; destination list is mutex-protected
- **Validation-first:** Messages fully parsed before forwarding
- **Checksum (Part 2):** Standard 16-bit one’s complement (like TCP/UDP)
- **Resilience:** Handles disconnections and poisoned mutexes gracefully

---

## 📜 License

This project is licensed under the MIT License 

---

## 📫 Contact

cjaycooper2@outlook.com
