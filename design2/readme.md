# Monte Carlo Energy Simulation for Off-Grid Northern Mines

## Overview

This project simulates energy generation, cost, and emissions for off-grid northern open-pit mines using a Monte Carlo approach. It combines historical weather data with random modeling of system parameters and diesel pricing to evaluate different energy system configurations.

---

## Setup Instructions

### 1. Create a Virtual Environment

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

Run the script:

```bash
python3 main.py
```

You will be prompted for two inputs:

### 1. Mine Size Scaling Factor

```
Mine size scaling factor:
```

* Represents the size of the mine relative to a baseline
* Example:

  * `1.0` → baseline mine
  * `2.0` → twice as large
  * `0.5` → half the size

---

### 2. Renewable Energy Ratio

```
Renewable ratio (0-1):
```

* Fraction of energy demand to be supplied by wind
* Example:

  * `0.3` → 30% renewable
  * `0.7` → 70% renewable

---

## What the Simulation Does

Each Monte Carlo trial:

1. Samples uncertain parameters

   * Turbine efficiency ($C_p$)
   * Air density ($\rho$)

2. Resamples weather data

   * Bootstrap sampling from historical hourly data

3. Adds wind uncertainty

   * Multiplicative noise on wind speed

4. Determines turbine count

   * Based on required renewable energy and average wind conditions

5. Simulates diesel price

   * Random Process with Brownian Motion

6. Runs hourly energy balance

   * Wind supplies the renewable portion
   * Diesel supplies the remaining demand

---

## Outputs

### Console Output

After simulation:

```
=== RESULTS ===
Avg Emissions: X.XXe+XX kg CO2
Avg Cost per kWh: X.XXXX CAD/kWh
```

---

### Saved Plot

A figure is saved as:

```
monte_carlo_results.png
```

It contains two subplots:

#### Emissions Distribution

* Unit: Million kg CO₂
* Shows variability across simulations

#### Cost Distribution

* Unit: CAD/kWh
* Represents economic variability

---

## Key Model Assumptions

* Wind power:
  $$
  P = \frac{1}{2} \rho A C_p v^3
  $$

* Turbine output is capped at rated power

* Diesel:

  * Fixed fuel consumption rate
  * Emissions proportional to fuel use

* Weather:

  * Sampled from historical distribution
  * Temporal correlation is not preserved

---

## Notes and Limitations

* No energy storage is modeled
* No grid or transmission constraints
* Wind sampling breaks time dependence
* Turbine siting and wake effects are not modeled
* Diesel pricing follows a stochastic process assumption

---
