use std::{fs, path::PathBuf, str::FromStr, time::Instant};

use bytes::Bytes;
use clap::Parser;
use primitive_types::H160;
use revm::{InMemoryDB, Return, TransactOut, TransactTo};

extern crate alloc;

/// Revolutionary EVM (revm) runner interface
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the hex contract code to deploy and run
    #[arg(long)]
    contract_code_path: PathBuf,

    /// Hex of calldata to use when calling the contract
    #[arg(long)]
    calldata: String,

    /// Number of times to run the benchmark
    #[arg(short, long, default_value_t = 1)]
    num_runs: u8,
}

const CALLER_ADDRESS: &str = "0x1000000000000000000000000000000000000001";

fn main() {
    let args = Args::parse();

    let caller_address = H160::from_str(CALLER_ADDRESS).unwrap();

    let contract_code: Bytes =
        hex::decode(fs::read_to_string(args.contract_code_path).expect("unable to open file"))
            .expect("could not hex decode contract code")
            .into();
    let calldata: Bytes = hex::decode(args.calldata)
        .expect("could not hex decode calldata")
        .into();

    // Set up the EVM with a database and create the contract
    let mut evm = revm::new();
    evm.database(InMemoryDB::default());
    evm.env.tx.caller = caller_address;
    evm.env.tx.transact_to = TransactTo::create();
    evm.env.tx.data = contract_code;
    let res = evm.transact_commit();
    match res.exit_reason {
        Return::Continue => {}
        reason => panic!("unexpected exit reason while creating: {:?}", reason),
    }
    let contract_address = match res.out {
        TransactOut::Create(_, Some(addr)) => addr,
        _ => panic!("could not get contract address"),
    };

    evm.env.tx.caller = caller_address;
    evm.env.tx.transact_to = TransactTo::Call(contract_address);
    evm.env.tx.data = calldata;

    for _ in 0..args.num_runs {
        let timer = Instant::now();
        let (res, _) = evm.transact();
        let dur = timer.elapsed();

        match res.exit_reason {
            Return::Return => (),
            reason => {
                panic!("unexpected exit reason while benchmarking: {:?}", reason)
            }
        }

        println!("{}", dur.as_millis())
    }
}
