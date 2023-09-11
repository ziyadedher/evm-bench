from typing import Final
from typing_extensions import Annotated

import time

import typer

from eth.constants import BLANK_ROOT_HASH, ZERO_HASH32, GAS_LIMIT_MAXIMUM, ZERO_ADDRESS
from eth.db.atomic import AtomicDB as DB
from eth.vm.execution_context import ExecutionContext
from eth.vm.message import Message
from eth.vm.transaction_context import BaseTransactionContext as TransactionContext
from eth.vm.forks.shanghai.computation import ShanghaiComputation as Computation
from eth.vm.forks.shanghai.state import ShanghaiState as State
from eth_typing import Address, BlockNumber
from eth_utils.hexadecimal import decode_hex


CALLER_ADDRESS: Final[str] = "0x1000000000000000000000000000000000000001"
CONTRACT_ADDRESS: Final[str] = "0x2000000000000000000000000000000000000002"


def main(
    contract_code: Annotated[str, typer.Option()],
    calldata: Annotated[str, typer.Option()],
    num_runs: Annotated[int, typer.Option()],
) -> None:
    caller = Address(decode_hex(CALLER_ADDRESS))
    contract_address = Address(decode_hex(CONTRACT_ADDRESS))
    calldata_bytes: bytes = decode_hex(calldata)
    contract_code_bytes = decode_hex(contract_code)

    state = State(
        DB(),
        ExecutionContext(ZERO_ADDRESS, 0, BlockNumber(0), 0, ZERO_HASH32, GAS_LIMIT_MAXIMUM, [], 0),
        BLANK_ROOT_HASH,
    )
    state.set_code(
        contract_address,
        decode_hex(contract_code),
    )

    message = Message(GAS_LIMIT_MAXIMUM, contract_address, caller, 0, calldata_bytes, contract_code_bytes)
    transaction_context = TransactionContext(0, caller)

    for _ in range(num_runs):
        start = time.perf_counter_ns()
        computation = Computation.apply_computation(
            state,
            message,
            transaction_context,
        )
        end = time.perf_counter_ns()
        assert computation.is_success
        print((end - start) / 1e3)


if __name__ == "__main__":
    typer.run(main)
