## Runners

evm-bench runners are platforms for deploying and calling arbitrary smart contracts.

Runner entry points should satisfy a simple interface and a few conditions to be able to consistently run different benchmarks. The evm-bench framework picks up on runners by scanning for `runner.evm-bench.json` files, which have [a schema](schema.json). That schema has more information on the structure of runner metadata file.

### Interface

The entry pointed to by the metadata file should be an executable that accepts three named command-line options:

- `--contract-code-path`: path to a compiled smart contract.
- `--call-data`: hexstring representing the calldata to use when calling the smart contract.
- `--num-runs`: integer number of runs to call the smart contract with the calldata.

Calling the entry point with valid arguments should output `num-runs` newline-separated number values representing, per line, the number of milliseconds that that particular run of the benchmark took.

### Conditions

To ensure a consistent and accurate benchmarking experience across runners, we have some sane conditions to follow for runners:

- Time _only_ the EVM interpreter loop.
  - In particular, avoid timing any database write, consensus, or block-building logic.
- Do not time the contract deployment.
  - Contract deployment is not part of the benchmark.
- Deploy the contract using the code loaded from the provided `contract-code-path`.
- Use the provided `calldata` to send a call transaction to the deployed contract.
- Call the contract exactly `num-runs` times.
- Output exactly `num-runs` lines, with a number value on each representing the millisecond time it took to execute each contract call.

### Developing a new runner

It all starts with choosing (or building) an EVM interpreter. This can be in any language or framework you'd like.

Once you have that, you need to build a shim that'll isolate the EVM loop and implement our runner interface. Check out [the source code for the `revm` runner](revm) for a straightforward implementation. In essence, make sure you implement the correct [runner interface](#interface) and follow the [conditions](#conditions). There isn't anything tricky about the interface.

Pay attention to how the runner will be built or run, though. Your entry point may use some toolchains to run or build your runner. Make sure any tools used are checked under the `validate_executable` calls in evm-bench's [`main.rs`](../src/main.rs).

All you need now is a new `runner.evm-bench.json` file somewhere under this directory (since this is where the tool scans for runners by default). Use the other runners here as an example! Create a new folder and add resources under that folder.

Once you have your runner, it's time to test! Consider running the evm-bench framework with a single benchmark ([`ten-thousand-hashes`](../benchmarks/ten-thousand-hashes) is the most stable in my experience) against your new runner to start, then move on to running all benchmarks. It would look something like `RUST_LOG=info cargo run -- --runners <my_new_runner_name> --benchmarks ten-thousand-hashes`, if you need more information about logs you can tweak `RUST_LOG`.
