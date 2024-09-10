import numpy as np
import json
import matplotlib.pyplot as plt
import matplotlib

matplotlib.use("Agg")

ENDPOINT_COLORS = {
    "Hash": "C0",
    "Train": "C1",
}


def InitializeMatplotLib():
    matplotlib.rc("font", family="serif", size=9)
    matplotlib.rc("text.latex", preamble="\\usepackage{times,mathptmx}")
    matplotlib.rc("text", usetex=True)
    matplotlib.rc("legend", fontsize=8)
    matplotlib.rc("figure", figsize=(3.63, 1.5))
    matplotlib.rc("axes", linewidth=0.5)
    matplotlib.rc("lines", linewidth=0.5)


# Parameters configuring parsing of inputs and legend of plot.
PLOT_LABELS = {
    "hash": "Hash User",
    "train": "Train Model",
}

ENDPOINTS = [
    "hash",
    "train",
    "hash_baseline",
    "train_baseline",
]


# Parse an input file.
def ParseWebsubmitFiles(dir):
    data = dict()

    for endpoint in ENDPOINTS:
        df = json.load(open(dir + "/" + endpoint + ".json"))
        data[endpoint] = df

    return data


def CalcAvgs(data):
    averages = []
    marks = ["Total", "Serialize", "Setup", "Function", "Teardown", "Deserialize", "Fold"]
    for (endpoint, bench) in data.items():
        print(endpoint)
        times = np.array(bench)
        times = np.mean(times, axis=0)
        averages.append((endpoint, times))
        for _ in range(len(times)):
            print("  ", marks[_], "\t\t", times[_], "micro")
        print("")

    return averages


def PlotBenchmarks(data):
    x = []
    serde = []
    setup_teardown = []
    func = []
    fold = []
    for (endpoint, times) in data:
        x.append(endpoint)
        serde.append(times[1] + times[5])
        setup_teardown.append(times[2] + times[4])
        func.append(times[3])
        fold.append(times[6])

    serde = np.array(serde)
    setup_teardown = np.array(setup_teardown)
    func = np.array(func)
    fold = np.array(fold)

    plt.rc('font', size=14)
    figure = plt.gcf().set_size_inches(8, 6)
    plt.bar(x, setup_teardown)
    plt.bar(x, serde, bottom=setup_teardown)
    plt.bar(x, fold, bottom=setup_teardown+serde)
    plt.bar(x, func, bottom=setup_teardown+serde+fold)
    plt.xlabel('Endpoints')
    plt.ylabel('Time (micro)')
    plt.legend(["Setup/Teardown", "Serde", "Fold", "Function"], fontsize=14)
    plt.savefig("benchmarks.png", bbox_inches="tight", pad_inches=0.5, dpi = 100)


# Main.
if __name__ == "__main__":
    InitializeMatplotLib()

    benches = ParseWebsubmitFiles("results")
    avgs = CalcAvgs(benches)
    PlotBenchmarks(avgs)
