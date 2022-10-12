import argparse
from typing import Final, cast

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


GAS_LIMIT: Final[int] = 1_000_000_000
ZERO_ADDRESS: Final[eth_typing.Address] = eth.constants.ZERO_ADDRESS

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


def _construct_chain() -> eth.chains.base.MiningChain:
    chain_class = eth.chains.base.MiningChain.configure(
        __name__="TestChain",
        vm_configuration=(
            (eth.constants.GENESIS_BLOCK_NUMBER, eth.vm.forks.berlin.BerlinVM),
        ),
    )
    chain = chain_class.from_genesis(
        eth.db.atomic.AtomicDB(),
        genesis_params={
            "difficulty": 100,
            "gas_limit": 2 * GAS_LIMIT,
        },
    )

    return cast(eth.chains.base.MiningChain, chain)


def _benchmark(
    chain: eth.chains.base.MiningChain,
    caller_address: eth_typing.Address,
    caller_private_key: eth_keys.keys.PrivateKey,
    contract_code: bytes,
    call_data: bytes,
    num_runs: int,
) -> None:
    evm = chain.get_vm()
    nonce = evm.state.get_nonce(caller_address)
    tx = evm.create_unsigned_transaction(
        nonce=nonce,
        gas_price=0,
        gas=GAS_LIMIT,
        to=eth.constants.CREATE_CONTRACT_ADDRESS,
        value=0,
        data=contract_code,
    )
    signed_tx = tx.as_signed_transaction(caller_private_key)
    _, computation = evm.apply_transaction(chain.header, signed_tx)

    contract_address = computation.msg.storage_address

    nonce = evm.state.get_nonce(caller_address)
    tx = evm.create_unsigned_transaction(
        nonce=nonce,
        gas_price=0,
        gas=GAS_LIMIT,
        to=contract_address,
        value=0,
        data=call_data,
    )
    signed_tx = tx.as_signed_transaction(caller_private_key)
    evm_message = evm.state.get_transaction_executor().build_evm_message(signed_tx)

    def bench() -> None:
        evm.state.get_transaction_executor().build_computation(evm_message, signed_tx)

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
    chain = _construct_chain()

    _benchmark(
        chain,
        caller_address=CALLER_ADDRESS,
        caller_private_key=CALLER_PRIVATE_KEY,
        contract_code=contract_data,
        call_data=bytes.fromhex(args.calldata),
        num_runs=args.num_runs,
    )


if __name__ == "__main__":
    main()
