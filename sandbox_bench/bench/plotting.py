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
]


# Parse an input file.
def ParseWebsubmitFiles(dir):
    data = dict()

    for endpoint in ENDPOINTS:
        df = json.load(open(dir + "/" + endpoint + ".json"))
        data[endpoint] = df

    return data


def CalcAvgs(data):
    avgs = []
    for (endpoint, bench) in data.items():
        times = np.array(bench)
        times = np.mean(times, axis=0)
        print(times)
        avgs.append((endpoint, times))

    return avgs


def PlotBenchmarks(data):
    x = []
    y1 = []
    y2 = []
    for (endpoint, times) in data:
        x.append(endpoint)
        y1.append(times[0])
        y2.append(times[1])
    plt.bar(x, y1, color='r')
    plt.bar(x, y2, bottom=y1, color='b')
    plt.xlabel('Endpoints')
    plt.ylabel('Time (ns)')
    plt.legend(["Construction", "Teardown"])
    plt.title("Sandbox Drill Down")
    plt.savefig("benchmarks.png", bbox_inches="tight", pad_inches=0.01)


# Main.
if __name__ == "__main__":
    InitializeMatplotLib()

    benches = ParseWebsubmitFiles("benches")
    avgs = CalcAvgs(benches)
    PlotBenchmarks(avgs)
