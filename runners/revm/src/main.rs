use std::{fs, path::PathBuf, str::FromStr, time::Instant};

use bytes::Bytes;
use clap::Parser;
use revm_interpreter::{
    analysis::to_analysed,
    primitives::{Bytecode, Env, LatestSpec, TransactTo, B160},
    Contract, DummyHost, InstructionResult, Interpreter,
};
//use revm-interpreter::{}

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

    let caller_address = B160::from_str(CALLER_ADDRESS).unwrap();

    let contract_code: Bytes =
        hex::decode(fs::read_to_string(args.contract_code_path).expect("unable to open file"))
            .expect("could not hex decode contract code")
            .into();
    let calldata: Bytes = hex::decode(args.calldata)
        .expect("could not hex decode calldata")
        .into();

    // Set up the EVM with a database and create the contract
    let mut env = Env::default();
    env.tx.caller = caller_address;
    env.tx.transact_to = TransactTo::create();
    env.tx.data = calldata.clone();

    let bytecode = to_analysed::<LatestSpec>(Bytecode::new_raw(contract_code));

    // revm interpreter. (rakita note: should be simplified in one of next version.)
    let contract = Contract::new_env::<LatestSpec>(&env, bytecode);
    let mut host = DummyHost::new(env.clone());
    let mut interpreter = Interpreter::new(contract, u64::MAX, false);
    let reason = interpreter.run::<_, LatestSpec>(&mut host);

    match reason {
        InstructionResult::Stop | InstructionResult::Return => {}
        reason => panic!("unexpected exit reason while creating: {:?}", reason),
    }
    let created_contract = interpreter.return_value();

    env.tx.caller = caller_address;
    env.tx.data = calldata;

    let created_bytecode = to_analysed::<LatestSpec>(Bytecode::new_raw(created_contract));
    let contract = Contract::new_env::<LatestSpec>(&env, created_bytecode);

    for _ in 0..args.num_runs {
        let mut interpreter = revm_interpreter::Interpreter::new(contract.clone(), u64::MAX, false);
        let timer = Instant::now();
        let reason = interpreter.run::<_, LatestSpec>(&mut host);
        let dur = timer.elapsed();
        host.clear();

        match reason {
            InstructionResult::Return | InstructionResult::Stop => (),
            reason => {
                panic!("unexpected exit reason while benchmarking: {:?}", reason)
            }
        }

        println!("{}", dur.as_micros() as f64 / 1e3)
    }
}
