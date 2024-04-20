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
    marks = ["Total", "Serialize", "Setup", "Function", "Teardown", "Deserialize"]
    for (endpoint, bench) in data.items():
        print("Endpoint: ", endpoint)
        times = np.array(bench)
        times = np.mean(times, axis=0)
        times = times / 1000
        for _ in range(len(times)):
            print(marks[_], " ", times[_])


def PlotBenchmarks(data):
    x = []
    y1 = []
    y2 = []
    y3 = []
    y4 = []
    y5 = []
    # y6 = []
    for (endpoint, times) in data:
        x.append(endpoint)
        y1.append(times[1])
        y2.append(times[2])
        y3.append(times[3])
        y4.append(times[4])
        y5.append(times[5])
        # y6.append(times[5])
    y1 = np.array(y1)
    y2 = np.array(y2)
    y3 = np.array(y3)
    y4 = np.array(y4)
    y5 = np.array(y5)
    plt.bar(x, y2)
    plt.bar(x, y1, bottom=y2)
    plt.bar(x, y3, bottom=y1+y2)
    plt.bar(x, y4, bottom=y1+y2+y3)
    plt.bar(x, y5, bottom=y1+y2+y3+y4)
    plt.xlabel('Endpoints')
    plt.ylabel('Time (ns)')
    plt.legend(["Setup", "Serialize", "Function", "Teardown", "Deserialize"])
    plt.savefig("benchmarks.png", bbox_inches="tight", pad_inches=0.01)


# Main.
if __name__ == "__main__":
    InitializeMatplotLib()

    benches = ParseWebsubmitFiles("results")
    CalcAvgs(benches)
    # PlotBenchmarks(avgs)
