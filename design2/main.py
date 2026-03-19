import numpy as np
import pandas as pd
import matplotlib.pyplot as plt
from tqdm import tqdm
import seaborn as sns
sns.set_theme()



RHO = 1.225
ROTOR_AREA = 3959
CP = 0.4
TURBINE_RATED_POWER_KW = 2300  # per turbine

FUEL_CONSUMPTION = 0.27  # L/kWh
CO2_PER_LITER = 2.68

K_AVG = 1.667  # MW/km^2 baseline

DIESEL_PRICE_INIT = 1.5  # CAD/L
DIESEL_MU = 0.0
DIESEL_SIGMA = 0.2



def classify_season(month):
    if month in [12, 1, 2]:
        return "winter"
    elif month in [3, 4, 5]:
        return "spring"
    elif month in [6, 7, 8]:
        return "summer"
    else:
        return "fall"

def wind_power(v, rho, cp, n_turbines):
    power = 0.5 * rho * ROTOR_AREA * cp * (v ** 3) * n_turbines  # W
    power_kw = power / 1000

    max_power = n_turbines * TURBINE_RATED_POWER_KW
    return min(power_kw, max_power)

def simulate_diesel_price_path(n_steps):
    prices = np.zeros(n_steps)
    prices[0] = DIESEL_PRICE_INIT
    dt = 1 / 8760

    for t in range(1, n_steps):
        z = np.random.normal()
        prices[t] = prices[t-1] * np.exp(
            (DIESEL_MU - 0.5 * DIESEL_SIGMA**2) * dt +
            DIESEL_SIGMA * np.sqrt(dt) * z
        )

    return prices

def sample_weather(df):
    return df.sample(n=len(df), replace=True).reset_index(drop=True)



def estimate_required_turbines(df, mine_scale, renewable_ratio, rho, cp):
    """
    Estimate turbine count based on average wind resource
    """

    demand_mw = mine_scale * K_AVG
    target_kwh = renewable_ratio * demand_mw * 1000  # per hour target

    # Use mean wind speed to estimate expected turbine output
    mean_v = df["WIND_SPEED"].mean()

    avg_power_per_turbine_kw = (
        0.5 * rho * ROTOR_AREA * cp * (mean_v ** 3)
    ) / 1000

    avg_power_per_turbine_kw = min(avg_power_per_turbine_kw, TURBINE_RATED_POWER_KW)

    if avg_power_per_turbine_kw <= 0:
        return 1

    n_turbines = np.ceil(target_kwh / avg_power_per_turbine_kw)

    return int(max(1, n_turbines))



def run_single_simulation(df, mine_scale, renewable_ratio):

    total_energy_kwh = 0
    total_cost = 0
    total_emissions = 0

    #uncertainty parameters
    cp = np.random.normal(CP, 0.05)
    rho = np.random.normal(RHO, 0.05)

    n_turbines = estimate_required_turbines(df, mine_scale, renewable_ratio, rho, cp)

    demand_mw = mine_scale * K_AVG

    #sample weather
    sampled_df = sample_weather(df)

    
    diesel_prices = simulate_diesel_price_path(len(sampled_df))

    for i, (_, row) in enumerate(sampled_df.iterrows()):

        wind_speed = row["WIND_SPEED"]

        if np.isnan(wind_speed):
            continue

        # Wind stochastic perturbation
        wind_speed = wind_speed * np.random.normal(1.0, 0.1)

        wind_kwh = wind_power(wind_speed, rho, cp, n_turbines)

        demand_kwh = demand_mw * 1000

        renewable_target = renewable_ratio * demand_kwh

        wind_used = min(wind_kwh, renewable_target)
        diesel_needed = demand_kwh - wind_used

        # Diesel cost
        diesel_price = diesel_prices[i]
        diesel_liters = diesel_needed * FUEL_CONSUMPTION
        cost_diesel = diesel_liters * diesel_price

        # Wind cost (proxy)
        cost_wind = wind_used * 0.05

        emissions = diesel_liters * CO2_PER_LITER

        total_energy_kwh += demand_kwh
        total_cost += cost_wind + cost_diesel
        total_emissions += emissions

    cost_per_kwh = total_cost / total_energy_kwh

    return total_emissions, cost_per_kwh



def monte_carlo(df, mine_scale, renewable_ratio, n_trials=100):

    emissions_results = []
    cost_results = []

    for _ in tqdm(range(n_trials), desc="Running Monte Carlo simulations"):
        e, c = run_single_simulation(df, mine_scale, renewable_ratio)
        emissions_results.append(e)
        cost_results.append(c)

    return np.array(emissions_results), np.array(cost_results)



def plot_results(emissions, costs, save_path="monte_carlo_results.png"):
    fig, axes = plt.subplots(1, 2, figsize=(12, 5))

    axes[0].hist(emissions / 1e6, bins=50)
    axes[0].set_title("Total Emissions (Million kg CO2)")
    axes[0].set_xlabel("Emissions")
    axes[0].set_ylabel("Frequency")

    axes[1].hist(costs, bins=50)
    axes[1].set_title("Cost per kWh (CAD)")
    axes[1].set_xlabel("Cost")
    axes[1].set_ylabel("Frequency")

    plt.tight_layout()
    plt.savefig(save_path, dpi=300)
    plt.close(fig)


def main():
    df = pd.read_csv("../weather/data.csv")

    if "LOCAL_MONTH" in df.columns:
        df["season"] = df["LOCAL_MONTH"].apply(classify_season)

    mine_scale = float(input("Mine size scaling factor: "))
    renewable_ratio = float(input("Renewable ratio (0-1): "))

    emissions, costs = monte_carlo(
        df,
        mine_scale=mine_scale,
        renewable_ratio=renewable_ratio,
        n_trials=100
    )


    print("\n=== RESULTS ===")
    print(f"Avg Emissions: {pd.Series(emissions).describe()} kg CO2")
    print(f"Avg Cost per kWh: {pd.Series(costs).describe()} CAD/kWh")
    plot_results(emissions, costs)

if __name__ == "__main__":
    main()
