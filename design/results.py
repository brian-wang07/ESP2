import pandas as pd
import numpy as np
import matplotlib.pyplot as plt
import seaborn as sns
sns.set_theme()

df = pd.read_csv("monte_carlo_results.csv")

fig, axes = plt.subplots(1, 3, figsize=(18, 5))

axes[0].hist(df["emissions"], bins=50)
axes[0].set_title("Emissions Intensity (kg CO2/kWh)")
axes[0].set_xlabel("Emissions Intensity (kg CO2/kWh)")
axes[0].set_ylabel("Frequency")

axes[1].hist(df["cost_per_year"] / 1e6, bins=50)
axes[1].set_title("Annual Cost (Million CAD)")
axes[1].set_xlabel("Cost (Million CAD/year)")
axes[1].set_ylabel("Frequency")

axes[2].hist(df["social_acceptance"], bins=np.linspace(0, 1, 30))
axes[2].set_title("Social Acceptance Score")
axes[2].set_xlabel("SA Score (0=low, 1=high)")
axes[2].set_ylabel("Frequency")

plt.tight_layout()
plt.savefig("monte_carlo_results.png", dpi=300)
