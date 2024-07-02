import argparse
import pathlib
import timeit
from typing import Final

from coincurve import PrivateKey
from ethereum.base_types import U64, U256, Bytes0, Bytes32, Uint
from ethereum.cancun.fork import process_transaction
from ethereum.cancun.fork_types import Address
from ethereum.cancun.state import State, TransientStorage
from ethereum.cancun.transactions import LegacyTransaction
from ethereum.cancun.utils.address import compute_contract_address
from ethereum.cancun.vm import Environment
from ethereum.crypto.hash import keccak256

ZERO: Final[Address] = Address(b"\0" * 20)
GAS_LIMIT: Final[Uint] = Uint(1_000_000_000_000_000_000)
SENDER_SECRET: Final[bytes] = bytes.fromhex(
    "45a915e4d060149eb4365960e6a7a45f334393093061116b197e3240065ff2d8"
)
SENDER_PRIVATE_KEY: Final[PrivateKey] = PrivateKey(SENDER_SECRET)
SENDER: Final[Address] = Address(
    keccak256(SENDER_PRIVATE_KEY.public_key.format(compressed=False)[1:])[12:32]
)


def _load_contract_data(data_file_path: pathlib.Path) -> bytes:
    with open(data_file_path, mode="r") as file:
        return bytes.fromhex(file.read())


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--contract-code-path", type=pathlib.Path)
    parser.add_argument("--calldata", type=str)
    parser.add_argument("--num-runs", type=int)
    return parser.parse_args()


def main() -> None:
    args = parse_args()

    contract_data = _load_contract_data(args.contract_code_path)
    calldata = bytes.fromhex(args.calldata)

    state = State()

    env = Environment(
        caller=SENDER,
        origin=SENDER,
        block_hashes=[],
        coinbase=ZERO,
        number=Uint(0),
        gas_limit=GAS_LIMIT,
        base_fee_per_gas=Uint(0),
        gas_price=Uint(0),
        time=U256(0),
        state=state,
        chain_id=U64(1),
        traces=[],
        prev_randao=Bytes32([0] * 32),
        excess_blob_gas=U64(0),
        blob_versioned_hashes=(),
        transient_storage=TransientStorage(),
    )

    tx = LegacyTransaction(
        nonce=U256(0),
        gas_price=Uint(0),
        gas=GAS_LIMIT,
        to=Bytes0(),
        value=U256(0),
        data=contract_data,
        v=U256(0),
        r=U256(0),
        s=U256(0),
    )

    gas_used, logs, error = process_transaction(env, tx)
    if error is not None:
        raise error

    contract = compute_contract_address(SENDER, Uint(tx.nonce))

    tx = LegacyTransaction(
        nonce=U256(0),
        gas_price=Uint(0),
        gas=GAS_LIMIT,
        to=contract,
        value=U256(0),
        data=calldata,
        v=U256(0),
        r=U256(0),
        s=U256(0),
    )

    def bench():
        gas_used, logs, error = process_transaction(env, tx)
        if error is not None:
            raise error

    results = timeit.repeat(bench, number=1, repeat=args.num_runs)

    for result in results:
        print(result * 1000)


if __name__ == "__main__":
    main()
