use std::{str::FromStr, time::Instant};

use bytes::Bytes;
use clap::Parser;
use revm_interpreter::{
    analysis::to_analysed,
    primitives::{Bytecode, Env, LatestSpec, TransactTo, B160},
    Contract, DummyHost, InstructionResult, Interpreter,
};

extern crate alloc;

/// revm runner interface
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Hex of the contract code to deploy and run
    #[arg(long)]
    contract_code: String,

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

    let caller_address = B160::from_str(CALLER_ADDRESS).unwrap();

    let contract_code: Bytes = hex::decode(args.contract_code)
        .expect("could not hex decode contract code")
        .into();
    let calldata: Bytes = hex::decode(args.calldata)
        .expect("could not hex decode calldata")
        .into();

    let mut env = Env::default();
    env.tx.caller = caller_address;
    env.tx.transact_to = TransactTo::create();
    env.tx.data = calldata;

    let bytecode = to_analysed(Bytecode::new_raw(contract_code));
    let bytecode_hash = bytecode.hash_slow();
    let contract = Box::new(Contract::new_env(&env, bytecode, bytecode_hash));

    let mut interpreter = Interpreter::new(contract, u64::MAX, false);
    let mut host: DummyHost = DummyHost::new(env);

    for _ in 0..args.num_runs {
        let timer = Instant::now();
        let reason = interpreter.run::<_, LatestSpec>(&mut host);
        let dur = timer.elapsed();

        match reason {
            InstructionResult::Return | InstructionResult::Stop => (),
            reason => {
                panic!("unexpected exit reason while benchmarking: {reason:?}")
            }
        }

        println!("{}", dur.as_micros());
    }
}
