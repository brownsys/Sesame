# Websubmit Experiment

This repo is responsible for running the websubmit experiment (left half of figure 9, and figure 10(c) in the paper).

## Setup

First, create the python virtual environment required for our plotting scripts.
```bash
python3 -m venv venv
. venv/bin/activate
pip install -r requirements.txt
deactivate
```

This repo builds `../websubmit_boxed_sandboxes` using default features resulting in maximum performance.
If `../websubmit_boxed_sandboxes` has been built earlier (e.g. because you ran the sandbox experiment harness at `../sandbox_harness`),
it might have been built without these features, resulting in an incompatible `.so`.
If you try to run this harness against such a `.so` file, you will get a segfault.

To fix the segfault, run `make clean` in this directory.

## Running

To run the experiment:
```bash
make all  # rebuilds the sandbox  (if the .so file doesnt exist) and runs benchmark against both websubmit and websubmit_boxed
make plot # plots results
```

## Plotting

The plotting script has the values for the right half of figure 9 (portfolio experiment) hardcoded in it.

If you re-ran the portfolio experiment and would like to update the numbers in the output plot,
make sure to modify `plotting.py` at lines  322 and 325 with the results acquired from `applications/portfolio/portfolio_harness`.
