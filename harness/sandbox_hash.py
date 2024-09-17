import matplotlib as mpl
import matplotlib.pyplot as plt
import numpy as np

mpl.rc('font', family='serif', size=9)
mpl.rc('text.latex', preamble='\\usepackage{times,mathptmx}')
mpl.rc('text', usetex=True)
mpl.rc('legend', fontsize=8)
mpl.rc('figure', figsize=(3.63, 1.5))
mpl.rc('axes', linewidth=0.5)
mpl.rc('lines', linewidth=0.5)

# Data
categories = ['No Sandbox', 'Non-Optimized', 'Optimized']
set_tear = [0, 81.62, 15.86]
copy = [0, 2.97, 3.52]
func = [1.24, 1.68, 1.68]

# Compute total heights
total = [func[i] + set_tear[i] + copy[i] for i in range(len(categories))]

# Create bar positions
x = np.arange(len(categories))

# Plot
fig = plt.figure(figsize=(4, 2))
gs = fig.add_gridspec(2, 1, hspace=0.1)

# Upper plot (with break)
ax1 = fig.add_subplot(gs[0])
ax1.bar(x, func, label='Function', color='C0')
ax1.bar(x, copy, bottom=func, label='Copy', color='C1')
ax1.bar(x, set_tear, bottom=np.array(func) +
        np.array(copy), label='Setup + Tear', color='C2')

# Adjust the upper plot limits and add break indication
ax1.set_ylim(70, 95)
ax1.set_yticks(np.arange(75, 95, 10))
ax1.set_xlabel('Sandbox Type')

# Lower plot
ax2 = fig.add_subplot(gs[1])
ax2.bar(x, func, label='Function', color='C0')
ax2.bar(x, copy, bottom=func, label='Copy', color='C1')
ax2.bar(x, set_tear, bottom=np.array(func) +
        np.array(copy), label='Set + Tear', color='C2')

# Adjust the lower plot limits and add break indication
ax2.set_ylim(0, 25)
ax2.set_yticks(np.arange(0, 25, 10))

# Set x-ticks and labels for the lower plot
ax2.set_xticks(x)
ax2.set_xticklabels(categories)

# Combine legends and place them in the upper plot
handles, labels = ax1.get_legend_handles_labels()
ax1.legend(handles, labels, loc='upper left', frameon=False)

# Hide x-axis on the upper plot
ax1.xaxis.set_visible(False)

# Line break.
d = .5  # proportion of vertical to horizontal extent of the slanted line
kwargs = dict(marker=[(-1, -d), (1, d)], markersize=10,
              linestyle="none", color='k', mec='k', mew=1, clip_on=False)
ax1.plot([0, 1], [0, 0], transform=ax1.transAxes, **kwargs)
ax2.plot([0, 1], [1, 1], transform=ax2.transAxes, **kwargs)

# Add a common y-axis title
fig.text(0.04, 0.5, 'Time [µs]', va='center',
         ha='center', rotation='vertical', fontsize=9)

plt.savefig("sandbox_hash.pdf", format="pdf",
            bbox_inches="tight", pad_inches=0.01)
