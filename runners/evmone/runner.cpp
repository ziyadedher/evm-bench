#include <evmone/evmone.h>
#include <stdlib.h>

#include <CLI/CLI.hpp>
#include <chrono>
#include <evmc/evmc.hpp>
#include <evmc/hex.hpp>
#include <evmc/mocked_host.hpp>
#include <fstream>
#include <iostream>
#include <string>

#include "build/_deps/evmone-src/lib/evmone/advanced_analysis.hpp"
#include "build/_deps/evmone-src/lib/evmone/advanced_execution.hpp"

using namespace evmc::literals;

constexpr int64_t GAS = INT64_MAX;
const auto ZERO_ADDRESS = 0x0000000000000000000000000000000000000000_address;
const auto CALLER_ADDRESS = 0x1000000000000000000000000000000000000001_address;
const auto CONTRACT_ADDRESS =
    0x2000000000000000000000000000000000000002_address;

void check_status(evmc_result result) {
  std::cerr << evmc_status_code_to_string(result.status_code) << std::endl;
  if (result.status_code != EVMC_SUCCESS) {
    exit(1);
  }
}

int main(int argc, char** argv) {
  std::string contract_code;
  std::string calldata;
  uint32_t num_runs;

  CLI::App app{"evmone runner"};
  app.add_option("--contract-code", contract_code,
                 "Hex code of contract to run")
      ->required();
  app.add_option("--calldata", calldata,
                 "Hex of calldata to use when calling the contract")
      ->required();
  app.add_option("--num-runs", num_runs, "Number of times to run the benchmark")
      ->required();

  CLI11_PARSE(app, argc, argv);

  evmc::bytes calldata_bytes;
  calldata_bytes.reserve(calldata.size() / 2);
  evmc::from_hex(calldata.begin(), calldata.end(),
                 std::back_inserter(calldata_bytes));

  evmc::bytes contract_code_bytes;
  contract_code_bytes.reserve(contract_code.size() / 2);
  evmc::from_hex(contract_code.begin(), contract_code.end(),
                 std::back_inserter(contract_code_bytes));

  const evmc::MockedAccount account{0, contract_code_bytes, evmc::bytes32{},
                                    evmc::uint256be{}};
  evmc::MockedHost host;
  host.accounts.insert_or_assign(CONTRACT_ADDRESS, account);
  const auto host_interface = host.get_interface();

  const auto analysis = evmone::advanced::analyze(
      evmc_revision::EVMC_LATEST_STABLE_REVISION, contract_code_bytes);

  evmc_message call_msg{};
  call_msg.kind = EVMC_CALL;
  call_msg.gas = GAS;
  call_msg.input_data = calldata_bytes.data();
  call_msg.input_size = calldata_bytes.size();
  call_msg.recipient = CONTRACT_ADDRESS;
  call_msg.sender = CALLER_ADDRESS;

  auto state = evmone::advanced::AdvancedExecutionState(
      call_msg, evmc_revision::EVMC_LATEST_STABLE_REVISION,
      host.get_interface(), (evmc_host_context*)&host, contract_code_bytes);

  for (int i = 0; i < num_runs; i++) {
    const auto start = std::chrono::steady_clock::now();
    const auto call_result = evmone::advanced::execute(state, analysis);
    const auto end = std::chrono::steady_clock::now();
    check_status(call_result);

    using namespace std::literals;
    std::cout << (end - start) / 1.us << std::endl;
  }
};
