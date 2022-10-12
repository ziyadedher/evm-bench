from typing import Final, cast

import argparse
import pathlib
import time

import eth.abc
import eth.consensus.pow
import eth.constants
import eth.chains.base
import eth.db.atomic
import eth.tools.builder.chain.builders
import eth.vm.forks.berlin
import eth_keys
import eth_typing
import eth_utils
import pyrevm

GAS_LIMIT: Final[int] = 1_000_000_000
ZERO_ADDRESS: Final[str] = "0x0000000000000000000000000000000000000000"

CALLER_PRIVATE_KEY: Final[eth_keys.keys.PrivateKey] = eth_keys.keys.PrivateKey(
    eth_utils.hexadecimal.decode_hex(
        "0x45a915e4d060149eb4365960e6a7a45f334393093061116b197e3240065ff2d8"
    )
)
CALLER_ADDRESS: Final[eth_typing.Address] = eth_typing.Address(
    CALLER_PRIVATE_KEY.public_key.to_canonical_address()
)


def _load_contract_data(data_file_path: pathlib.Path) -> bytes:
    with open(data_file_path, mode="r") as file:
        return bytes.fromhex(file.read())


def _construct_evm() -> pyrevm.EVM:
    evm = pyrevm.EVM()
    return evm


def _benchmark(
    evm: pyrevm.EVM,
    caller_address: eth_typing.Address,
    caller_private_key: eth_keys.keys.PrivateKey,
    contract_data: bytes,
    call_data: list[int],
    num_runs: int,
) -> None:
    chain_class = eth.chains.base.MiningChain.configure(
        __name__="TestChain",
        vm_configuration=(
            (eth.constants.GENESIS_BLOCK_NUMBER, eth.vm.forks.berlin.BerlinVM),
        ),
    )
    chain = cast(
        eth.chains.base.MiningChain,
        chain_class.from_genesis(
            eth.db.atomic.AtomicDB(),
            genesis_params={
                "difficulty": 100,
                "gas_limit": 2 * GAS_LIMIT,
            },
        ),
    )
    pyevm_evm = chain.get_vm()
    nonce = pyevm_evm.state.get_nonce(caller_address)
    tx = pyevm_evm.create_unsigned_transaction(
        nonce=nonce,
        gas_price=0,
        gas=GAS_LIMIT,
        to=eth.constants.CREATE_CONTRACT_ADDRESS,
        value=0,
        data=contract_data,
    )
    signed_tx = tx.as_signed_transaction(caller_private_key)
    _, computation = pyevm_evm.apply_transaction(chain.header, signed_tx)

    contract_address = computation.msg.storage_address
    # assert computation.msg.code == pyevm_evm.state.get_code(contract_address)
    evm.insert_account_info(
        eth_utils.hexadecimal.encode_hex(contract_address),
        pyrevm.AccountInfo(code=pyevm_evm.state.get_code(contract_address)),
    )

    def bench() -> None:
        evm.call_raw(
            caller=eth_utils.hexadecimal.encode_hex(caller_address),
            to=eth_utils.hexadecimal.encode_hex(contract_address),
            data=call_data,
        )

    for _ in range(num_runs):
        start = time.perf_counter_ns()
        bench()
        end = time.perf_counter_ns()
        print((end - start) / 1e6)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--contract-code-path", type=pathlib.Path)
    parser.add_argument("--calldata", type=str)
    parser.add_argument("--num-runs", type=int)
    return parser.parse_args()


def main() -> None:
    args = parse_args()

    contract_data = _load_contract_data(args.contract_code_path)
    evm = _construct_evm()

    _benchmark(
        evm,
        caller_address=CALLER_ADDRESS,
        caller_private_key=CALLER_PRIVATE_KEY,
        contract_data=contract_data,
        call_data=list(bytes.fromhex(args.calldata)),
        num_runs=args.num_runs,
    )


if __name__ == "__main__":
    main()
