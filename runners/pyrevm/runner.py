from typing import Final
from typing_extensions import Annotated

import time

import eth_utils
import pyrevm
import typer

CALLER_ADDRESS: Final[str] = "0x1000000000000000000000000000000000000001"
CONTRACT_ADDRESS: Final[str] = "0x2000000000000000000000000000000000000002"


def main(
    contract_code: Annotated[str, typer.Option()],
    calldata: Annotated[str, typer.Option()],
    num_runs: Annotated[int, typer.Option()],
) -> None:
    caller = CALLER_ADDRESS
    to = CONTRACT_ADDRESS
    data = eth_utils.hexadecimal.decode_hex(calldata)

    evm = pyrevm.EVM()
    evm.insert_account_info(
        CONTRACT_ADDRESS,
        pyrevm.AccountInfo(code=eth_utils.hexadecimal.decode_hex(contract_code)),
    )

    for _ in range(num_runs):
        start = time.perf_counter_ns()
        evm.call_raw(caller=caller, to=to, data=data)
        end = time.perf_counter_ns()
        print((end - start) / 1e3)


if __name__ == "__main__":
    typer.run(main)
