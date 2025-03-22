# Websubmit Sandbox Drilldown Experiment

This repo is responsible for running the sandbox performance drill down experiment (similar to figure 10(a) and (b) in the paper, although the paper plots the numbers differently).

## Setup

First, create the python virtual environment required for our plotting scripts.
```bash
python3 -m venv venv
. venv/bin/activate
pip install -r requirements.txt
deactivate
```

This repo builds `../websubmit_boxed_sandboxes` using custom features for timing the sandbox internals.
If `../websubmit_boxed_sandboxes` has been built earlier (e.g. because you ran the regular websubmit harness at `../websubmit`),
it might have been built without these features, resulting in an incompatible `.so`.
If you try to run this harness against such a `.so` file, you will get a segfault.

To fix the segfault, run `make clean` in this directory.

## Running

To run the experiment:
```bash
make run  # rebuilds the sandbox  (if the .so file doesnt exist) and runs benchmark
make plot # plots results
```
