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
    "answer_questions_bench": "Answer Questions",
    "get_aggregates_bench": "Get Aggregates",
    "get_employer_info_bench": "Get Employer Info",
    "predict_grades_bench": "Predict Grades",
    "register_users_bench": "Register Users",
    "submit_grades_bench": "Submit grades",
    "view_answers_bench": "View Answers",
}
ENDPOINTS = ["answer_questions_bench", "get_aggregates_bench", "get_employer_info_bench", "predict_grades_bench",
             "register_users_bench", "submit_grades_bench", "view_answers_bench"]
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
    plt.ylim(ymax=15)
    plt.legend(frameon=False)
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

    return data


# Main.
if __name__ == "__main__":
    InitializeMatplotLib()

    # Parse input data.
    baseline = ParseWebsubmitFiles('benches')
    alohomora = ParseWebsubmitBoxedFiles('benches')

    # Plot output.
    PlotMergedPercentiles(baseline, alohomora)
