import pandas as pd
import numpy as np

import matplotlib.pyplot as plt
import matplotlib

matplotlib.use("Agg")

SYSTEM_COLORS = {
    'Baseline': 'C0',
    'With Alohomora': 'C1',
}


def InitializeMatplotLib():
    matplotlib.rc('font', family='serif', size=9)
    matplotlib.rc('text.latex', preamble='\\usepackage{times,mathptmx}')
    matplotlib.rc('text', usetex=True)
    matplotlib.rc('legend', fontsize=8)
    matplotlib.rc('figure', figsize=(3.63, 1.5))
    matplotlib.rc('axes', linewidth=0.5)
    matplotlib.rc('lines', linewidth=0.5)


# Parameters configuring parsing of inputs and legend of plot.
PLOT_LABELS = {
    "register_users_bench": "Register Users",
    "answer_questions_bench": "Answer Questions",
    "view_answers_bench": "View Answers",
    "submit_grades_bench": "Submit Grades",
    "retrain_model_bench": "Retrain Model",
    "predict_grades_bench": "Predict Grades",
    "get_aggregates_bench": "Get Aggregates",
    "get_employer_info_bench": "Get Employer Info",
}
ENDPOINTS = [
    "register_users_bench",
    "answer_questions_bench",
    "view_answers_bench",
    "submit_grades_bench",
    "retrain_model_bench",
    "predict_grades_bench",
    "get_aggregates_bench",
    "get_employer_info_bench",
]
PERCENTILES = ["50", "95"]

X = np.arange(len(ENDPOINTS))
W = 0.3


# Plot 50th and 95th percentile on one figure
def PlotMergedPercentiles(baseline, alohomora):
    for percentile in PERCENTILES:
        b = [baseline[endpoint][percentile] for endpoint in ENDPOINTS]
        a = [alohomora[endpoint][percentile] for endpoint in ENDPOINTS]

        alpha = 1 if percentile == "50" else 0.3
        label_baseline = "Baseline" if percentile == "50" else None
        label_alohomora = "With Alohomora" if percentile == "50" else None

        plt.bar(X - 0.5 * W, b, W, label=label_baseline,
                color=SYSTEM_COLORS['Baseline'], alpha=alpha)
        plt.bar(X + 0.5 * W, a, W, label=label_alohomora,
                color=SYSTEM_COLORS['With Alohomora'], alpha=alpha)

    plt.ylabel("Latency [ms]")
    plt.xticks(X, [PLOT_LABELS[e] for e in ENDPOINTS], rotation=25, ha='right')
    plt.xlabel("Websubmit Comparison")
    plt.ylim(ymax=45)
    plt.legend(frameon=False)
    plt.savefig("websubmit.pdf", format="pdf",
                bbox_inches="tight", pad_inches=0.01)

# Plot 50th and 95th percentile on one figure
def PlotMeanAndStd(baseline, alohomora):
    b_mean = [baseline[endpoint]['mean'] for endpoint in ENDPOINTS]
    a_mean = [alohomora[endpoint]['mean'] for endpoint in ENDPOINTS]

    b_std = [baseline[endpoint]['std'] for endpoint in ENDPOINTS]
    a_std = [alohomora[endpoint]['std'] for endpoint in ENDPOINTS]

    label_baseline = "Baseline"
    label_alohomora = "With Alohomora"

    plt.errorbar(X - 0.5 * W, b_mean, yerr=b_std, label=label_baseline,
            color=SYSTEM_COLORS['Baseline'], linestyle='None', marker='o', markersize=1)
    plt.errorbar(X + 0.5 * W, a_mean, yerr=a_std, label=label_alohomora,
            color=SYSTEM_COLORS['With Alohomora'], linestyle='None', marker='o', markersize=1)

    plt.ylabel("Latency [ms]")
    plt.xticks(X, [PLOT_LABELS[e] for e in ENDPOINTS], rotation=25, ha='right')
    plt.xlabel("Websubmit Comparison")
    plt.ylim(ymax=45)
    plt.legend(frameon=False, loc='upper left')
    plt.savefig("websubmit.pdf", format="pdf",
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


# Main.
if __name__ == "__main__":
    InitializeMatplotLib()

    # Parse input data.
    baseline = ParseWebsubmitFiles('benches')
    alohomora = ParseWebsubmitBoxedFiles('benches')

    # Plot output.
    PlotMergedPercentiles(baseline, alohomora)
    # PlotMeanAndStd(baseline, alohomora)
