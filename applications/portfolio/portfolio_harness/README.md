# Portfolio Experiment

This repo is responsible for running the portfolio experiment (right half of figure 9).

## Setup

This repo builds `../portfolio_boxed/sandbox`, if you modified the sandbox code and would like
to build and link against the new code, run `make clean` in this directory prior to rebuilding.

## Running

To run the experiment:
```bash
make boxed
make unboxed
```

## Plotting

Figure 9 in the paper contains runtimes from both this experiment and the websubmit experiment.

To reproduce that figure, also run the websubmit experiment by following instructions in
`applications/websubmit/harness/README.md`.

Then, modify `applications/websubmit/harness/plotting.py` as explained in the above README with
the new numbers that you see outputed to the terminal after running `make boxed` and `make unboxed`.

Finally, you can run the plotting script in the websubmit harness to get the plot.
