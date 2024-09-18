import pandas as pd
import numpy as np

import matplotlib.pyplot as plt
import matplotlib

matplotlib.use("Agg")

SYSTEM_COLORS = {
    'Baseline': 'C0',
    'Sesame': 'C1',
    'Naive Sesame': 'C2',
}


def InitializeMatplotLib():
    matplotlib.rc('font', family='serif', size=10)
    matplotlib.rc('text.latex', preamble='\\usepackage{times,mathptmx}')
    matplotlib.rc('text', usetex=True)
    matplotlib.rc('legend', fontsize=8)
    matplotlib.rc('figure', figsize=(3.63, 1.5))
    matplotlib.rc('axes', linewidth=0.5)
    matplotlib.rc('lines', linewidth=0.5)


# Parameters configuring parsing of inputs and legend of plot.
PLOT_LABELS = {
    "register_users_bench": "Register Users",
    "retrain_model_bench": "Retrain Model",
    "predict_grades_bench": "Predict Grades",
    "get_aggregates_bench": "Get Aggregates",
    "get_employer_info_bench": "Get Employer Info",
}

ENDPOINTS = [
    "register_users_bench",
    "retrain_model_bench",
    "predict_grades_bench",
    "get_aggregates_bench",
    "get_employer_info_bench",
]

FOLD_BASELINE_ENDPOINTS = [
    "view_answers_bench",
    "get_discussion_leader_bench",
]

FOLD_ALOHOMORA_ENDPOINTS = [
    "boxed_view_answers_bench",
    "boxed_get_discussion_leader_bench",
]

FOLD_NAIVE_ALOHOMORA_ENDPOINTS = [
    "boxed_view_answers_naive_bench",
    "boxed_get_discussion_leader_naive_bench",
]

FOLD_ENDPOINTS = [
    "Admin",
    "Discussion Leader",
]

PERCENTILES = ["50", "95"]

X = np.arange(len(ENDPOINTS))
X_F = np.arange(len(FOLD_ENDPOINTS))
W = 0.3


# Plot 50th and 95th percentile on one figure
def PlotMergedPercentiles(baseline, alohomora):
    fig, (ax1, ax2) = plt.subplots(2, 1, sharex=True)
    fig.subplots_adjust(hspace=0.1)

    ax1.set_ylim(0.09, 25)
    ax2.set_ylim(0, 0.09)

    for percentile in PERCENTILES:
        b = [baseline[endpoint][percentile] for endpoint in ENDPOINTS]
        a = [alohomora[endpoint][percentile] for endpoint in ENDPOINTS]

        alpha = 1 if percentile == "50" else 0.3
        label_baseline = "Baseline" if percentile == "50" else None
        label_alohomora = "Sesame" if percentile == "50" else None

        ax1.bar(X - 0.5 * W, b, W, label=label_baseline,
                color=SYSTEM_COLORS['Baseline'], alpha=alpha)
        ax2.bar(X - 0.5 * W, b, W, label=label_baseline,
                color=SYSTEM_COLORS['Baseline'], alpha=alpha)

        ax1.bar(X + 0.5 * W, a, W, label=label_alohomora,
                color=SYSTEM_COLORS['Sesame'], alpha=alpha)
        ax2.bar(X + 0.5 * W, a, W, label=label_alohomora,
                color=SYSTEM_COLORS['Sesame'], alpha=alpha)

    d = .5  # proportion of vertical to horizontal extent of the slanted line
    kwargs = dict(marker=[(-1, -d), (1, d)], markersize=10,
                linestyle="none", color='k', mec='k', mew=1, clip_on=False)
    ax1.plot([0, 1], [0, 0], transform=ax1.transAxes, **kwargs)
    ax2.plot([0, 1], [1, 1], transform=ax2.transAxes, **kwargs)

    ax1.legend(frameon=False, fontsize=8)

    ax1.xaxis.set_ticks_position('none')
    ax2.set_xticks(X, [PLOT_LABELS[e] for e in ENDPOINTS], rotation=15, ha='right')

    # plt.xlabel("Websubmit Comparison")
    plt.ylabel("Latency [ms]")
    plt.savefig("websubmit.pdf", format="pdf",
                bbox_inches="tight", pad_inches=0.01)

# Plot 50th and 95th percentile on one figure
def PlotMergedPercentilesNoBreak(baseline, alohomora):
    fig = plt.figure(figsize=(3.33, 1.3))
    for percentile in PERCENTILES:
        b = [baseline[endpoint][percentile] for endpoint in ENDPOINTS]
        a = [alohomora[endpoint][percentile] for endpoint in ENDPOINTS]

        print("at percentile " + percentile + "\n");
        print("\n baseline train is " + str(b) + "\n");
        print("\n alohomora train is " + str(a) + "\n");

        alpha = 1 if percentile == "50" else 0.3
        label_baseline = "Baseline" if percentile == "50" else None
        label_alohomora = "Sesame" if percentile == "50" else None

        plt.bar(X - 0.5 * W, b, W, label=label_baseline,
                color=SYSTEM_COLORS['Baseline'], alpha=alpha)
        plt.bar(X + 0.5 * W, a, W, label=label_alohomora,
                color=SYSTEM_COLORS['Sesame'], alpha=alpha)

    plt.ylabel("Latency [ms]", loc="center")
    plt.xticks(X, [PLOT_LABELS[e] for e in ENDPOINTS], rotation=15, ha='right')
    # plt.xlabel("Websubmit Comparison")
    plt.ylim(ymax=11)
    plt.legend(frameon=False, fontsize=8)
    plt.savefig("websubmit.pdf", format="pdf",
                bbox_inches="tight", pad_inches=0.01)

def PlotFoldPercentiles(baseline, alohomora, naive):
    matplotlib.rc('figure', figsize=(2.22, 1.4))
    fig, (ax1, ax2) = plt.subplots(2, 1, sharex=True, figsize=(2.22, 1.4))
    fig.subplots_adjust(hspace=0.1)

    ax1.set_ylim(245, 265)  # outliers only
    ax2.set_ylim(0, 25)  # most of the data

    for percentile in PERCENTILES:
        b = [fold_baseline[endpoint][percentile] for endpoint in FOLD_BASELINE_ENDPOINTS]
        a = [fold_alohomora[endpoint][percentile] for endpoint in FOLD_ALOHOMORA_ENDPOINTS]
        n = [fold_naive[endpoint][percentile] for endpoint in FOLD_NAIVE_ALOHOMORA_ENDPOINTS]

        print("at percentile " + percentile + "\n");
        print("\n baseline train is " + str(b) + "\n");
        print("\n alohomora train is " + str(a) + "\n");
        print("\n naive alohomora train is " + str(n) + "\n");

        alpha = 1 if percentile == "50" else 0.3
        label_baseline = "Baseline" if percentile == "50" else None
        label_alohomora = "Sesame" if percentile == "50" else None
        label_naive = "Na\\\"{i}ve Sesame" if percentile == "50" else None

        ax1.bar(X_F - W, b, W, color=SYSTEM_COLORS['Baseline'], label=label_baseline, alpha=alpha)
        ax1.bar(X_F, n, W, color=SYSTEM_COLORS['Naive Sesame'], label=label_naive, alpha=alpha)
        ax1.bar(X_F + W, a, W, color=SYSTEM_COLORS['Sesame'], label=label_alohomora, alpha=alpha)

        ax2.bar(X_F - W, b, W, color=SYSTEM_COLORS['Baseline'], label=label_baseline, alpha=alpha)
        ax2.bar(X_F, n, W, color=SYSTEM_COLORS['Naive Sesame'], label=label_naive, alpha=alpha)
        ax2.bar(X_F + W, a, W, color=SYSTEM_COLORS['Sesame'], label=label_alohomora, alpha=alpha)

    d = .5  # proportion of vertical to horizontal extent of the slanted line
    kwargs = dict(marker=[(-1, -d), (1, d)], markersize=10,
                linestyle="none", color='k', mec='k', mew=1, clip_on=False)
    ax1.plot([0, 1], [0, 0], transform=ax1.transAxes, **kwargs)
    ax2.plot([0, 1], [1, 1], transform=ax2.transAxes, **kwargs)

    ax1.legend(frameon=False, loc='upper left',
               borderaxespad=0.2, handletextpad=0.2, handlelength=1)

    ax1.xaxis.set_ticks_position('none')
    ax2.set_yticks([0, 10, 20])
    ax2.set_xticks(X_F, FOLD_ENDPOINTS, rotation=15, ha='right')

    # plt.xlabel("Fold Comparison")
    plt.subplots_adjust(left=0.22)
    fig.text(0, 0.5, 'Latency [ms]', va='center', rotation='vertical')

    plt.savefig("fold.pdf", format="pdf",
                bbox_inches="tight", pad_inches=0.01)
    matplotlib.rc('figure', figsize=(3.63, 1.5))

# Plot 50th and 95th percentile on one figure
def PlotMeanAndStd(baseline, alohomora):
    b_mean = [baseline[endpoint]['mean'] for endpoint in ENDPOINTS]
    a_mean = [alohomora[endpoint]['mean'] for endpoint in ENDPOINTS]

    b_std = [baseline[endpoint]['std'] for endpoint in ENDPOINTS]
    a_std = [alohomora[endpoint]['std'] for endpoint in ENDPOINTS]

    label_baseline = "Baseline"
    label_alohomora = "Sesame"

    plt.errorbar(X - 0.5 * W, b_mean, yerr=b_std, label=label_baseline,
            color=SYSTEM_COLORS['Baseline'], linestyle='None', marker='o', markersize=1)
    plt.errorbar(X + 0.5 * W, a_mean, yerr=a_std, label=label_alohomora,
            color=SYSTEM_COLORS['Sesame'], linestyle='None', marker='o', markersize=1)

    plt.ylabel("Latency [ms]")
    plt.xticks(X, [PLOT_LABELS[e] for e in ENDPOINTS], rotation=25, ha='right')
    # plt.xlabel("Websubmit Comparison")
    plt.ylim(ymax=20)
    plt.legend(frameon=False, loc='upper left')
    plt.savefig("websubmit-mean.pdf", format="pdf",
                bbox_inches="tight", pad_inches=0.01)

# Parse an input file.
def ParseWebsubmitFiles(dir):
    data = dict()

    for endpoint in ENDPOINTS:
        df = pd.read_json(dir + "/" + endpoint + ".json")[0] / 1000000

        data[endpoint] = dict()
        data[endpoint]["50"] = np.quantile(df.to_numpy(), 0.5)
        data[endpoint]["95"] = np.quantile(df.to_numpy(), 0.95)
        data[endpoint]["mean"] = np.mean(df.to_numpy())
        data[endpoint]["std"] = np.std(df.to_numpy())

    return data


# Parse an input file.
def ParseWebsubmitBoxedFiles(dir):
    data = dict()

    for endpoint in ENDPOINTS:
        df = pd.read_json(dir + "/" + "boxed_" +
                          endpoint + ".json")[0] / 1000000

        data[endpoint] = dict()
        data[endpoint]["50"] = np.quantile(df.to_numpy(), 0.5)
        data[endpoint]["95"] = np.quantile(df.to_numpy(), 0.95)
        data[endpoint]["mean"] = np.mean(df.to_numpy())
        data[endpoint]["std"] = np.std(df.to_numpy())

    return data

# Parse an input file.
def ParseFoldWebsubmitFiles(dir):
    data = dict()

    for endpoint in FOLD_BASELINE_ENDPOINTS:
        df = pd.read_json(dir + "/" + endpoint + ".json")[0] / 1000000

        data[endpoint] = dict()
        data[endpoint]["50"] = np.quantile(df.to_numpy(), 0.5)
        data[endpoint]["95"] = np.quantile(df.to_numpy(), 0.95)
        data[endpoint]["mean"] = np.mean(df.to_numpy())
        data[endpoint]["std"] = np.std(df.to_numpy())

    return data

def ParseFoldWebsubmitBoxedFiles(dir):
    data = dict()

    for endpoint in FOLD_ALOHOMORA_ENDPOINTS:
        df = pd.read_json(dir + "/" + endpoint + ".json")[0] / 1000000

        data[endpoint] = dict()
        data[endpoint]["50"] = np.quantile(df.to_numpy(), 0.5)
        data[endpoint]["95"] = np.quantile(df.to_numpy(), 0.95)
        data[endpoint]["mean"] = np.mean(df.to_numpy())
        data[endpoint]["std"] = np.std(df.to_numpy())

    return data

def ParseFoldWebsubmitNaiveFiles(dir):
    data = dict()

    for endpoint in FOLD_NAIVE_ALOHOMORA_ENDPOINTS:
        df = pd.read_json(dir + "/" + endpoint + ".json")[0] / 1000000

        data[endpoint] = dict()
        data[endpoint]["50"] = np.quantile(df.to_numpy(), 0.5)
        data[endpoint]["95"] = np.quantile(df.to_numpy(), 0.95)
        data[endpoint]["mean"] = np.mean(df.to_numpy())
        data[endpoint]["std"] = np.std(df.to_numpy())

    return data

# Main.
if __name__ == "__main__":
    InitializeMatplotLib()

    # Parse input data.
    baseline = ParseWebsubmitFiles('benches')
    alohomora = ParseWebsubmitBoxedFiles('benches')

    fold_baseline = ParseFoldWebsubmitFiles('benches')
    fold_alohomora = ParseFoldWebsubmitBoxedFiles('benches')
    fold_naive = ParseFoldWebsubmitNaiveFiles('benches')

    # Plot output.
    # PlotMergedPercentiles(baseline, alohomora)
    PlotMergedPercentilesNoBreak(baseline, alohomora)
    # PlotMeanAndStd(baseline, alohomora)
    PlotFoldPercentiles(fold_baseline, fold_alohomora, fold_naive)
