# Design 3: Concurrent Rust-based Monte Carlo Simulation

## Overview

This design implements a Monte Carlo simluation for the same off-grid northern mines. However, by replacing the time-dependent brownian motion model for diesel pricing with one that is independent, concurrency is now possible, which dramatically increases performance at the cost of accuracy of the cost metric.

---
## Setup Instructions

### 1. Install Rust

#### **Windows (PowerShell)**
Install Rust via [this link](https://rust-lang.org/tools/install/)
- Click on "Download rustup-init.exe".

#### **Linux / macOS**

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

---

### 2. Set up Virtual Environment

#### **Windows (PowerShell)**

```bash
python -m venv venv
venv\Scripts\activate
```

#### **Linux / macOS**

```bash
python3 -m venv venv
source venv/bin/activate
```

---


### 2. Install Dependencies

Install all required packages using:

```bash
pip install -r requirements.txt
```

---

## Running the Simulation
For best performance, run the simulation using the following:
```bash
cargo run --release
```
Refer to the readme.md in design2/ for further instructions.

## Output

A .csv file will be saved as
```bash
monte_carlo_results.csv
```

To generate plots, run
```bash
python3 results.py
```
Which will save the resulting plots as monte_carlo_results.png.

After you are done, run `deactivate` to disable the virtual environment.
