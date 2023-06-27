#include <evmone/evmone.h>
#include <CLI/CLI.hpp>
#include <evmc/evmc.hpp>
#include <evmc/hex.hpp>
#include <evmc/mocked_host.hpp>

#include <stdlib.h>
#include <chrono>
#include <fstream>
#include <iostream>
#include <string>

using namespace evmc::literals;

constexpr int64_t GAS = INT64_MAX;
const auto ZERO_ADDRESS = 0x0000000000000000000000000000000000000000_address;
const auto CONTRACT_ADDRESS =
    0x2000000000000000000000000000000000000002_address;
const auto CALLER_ADDRESS = 0x1000000000000000000000000000000000000001_address;

void check_status(evmc_result result) {
  std::cerr << evmc_status_code_to_string(result.status_code) << std::endl;
  if (result.status_code != EVMC_SUCCESS) {
    exit(1);
  }
}

int main(int argc, char** argv) {
  std::string contract_code_path;
  std::string calldata;
  uint num_runs;

  CLI::App app{"evmone runner"};
  app.add_option("--contract-code-path", contract_code_path,
                 "Path to the hex contract code to deploy and run")
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

  const auto vm = evmc_create_evmone();
  evmc::MockedHost host;

  std::string contract_code_hex;
  std::ifstream file(contract_code_path);
  file >> contract_code_hex;
  evmc::bytes contract_code;
  contract_code.reserve(contract_code_hex.size() / 2);
  evmc::from_hex(contract_code_hex.begin(), contract_code_hex.end(),
                 std::back_inserter(contract_code));

  evmc_message create_msg{};
  create_msg.kind = evmc_call_kind::EVMC_CREATE;
  create_msg.recipient = CONTRACT_ADDRESS;
  create_msg.gas = GAS;

  auto create_result =
      evmc_execute(vm, &host.get_interface(), (evmc_host_context*)&host,
                   evmc_revision::EVMC_LATEST_STABLE_REVISION, &create_msg,
                   contract_code.data(), contract_code.size());
  check_status(create_result);

  const auto& exec_code =
      evmc::bytes(create_result.output_data, create_result.output_size);

  evmc_message call_msg{};
  call_msg.kind = EVMC_CALL;
  call_msg.gas = GAS;
  call_msg.input_data = calldata_bytes.data();
  call_msg.input_size = calldata_bytes.size();
  call_msg.recipient = CONTRACT_ADDRESS;
  call_msg.sender = CALLER_ADDRESS;

  for (int i = 0; i < num_runs; i++) {
    evmc::MockedHost host;
    auto start = std::chrono::steady_clock::now();
    auto call_result =
        evmc_execute(vm, &host.get_interface(), (evmc_host_context*)&host,
                     evmc_revision::EVMC_LATEST_STABLE_REVISION, &call_msg,
                     exec_code.data(), exec_code.size());
    auto end = std::chrono::steady_clock::now();
    check_status(call_result);

    using namespace std::literals;
    std::cout << (end - start) / 1.ms << std::endl;
  }
};
