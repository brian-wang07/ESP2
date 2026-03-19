import pandas as pd
import matplotlib.pyplot as plt
import seaborn as sns
sns.set_theme()

df = pd.read_csv("monte_carlo_results.csv")

fig, axes = plt.subplots(1, 2, figsize=(12, 5))

axes[0].hist(df["emissions"] / 1e6, bins=50)
axes[0].set_title("Total Emissions (Million kg CO2)")
axes[0].set_xlabel("Emissions")
axes[0].set_ylabel("Frequency")

axes[1].hist(df["cost_per_kwh"], bins=50)
axes[1].set_title("Cost per kWh (CAD)")
axes[1].set_xlabel("Cost")
axes[1].set_ylabel("Frequency")

plt.tight_layout()
plt.savefig("monte_carlo_results.png", dpi=300)
plt.show()