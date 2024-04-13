import pandas as pd
import sys

df = pd.read_json(sys.argv[1])[0] / 1000

print("\tdelta [µs]")
percentile_values = df.quantile([.5, .95, .99])
print(percentile_values)
