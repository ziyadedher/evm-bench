use std::time::Instant;

use clap::Parser;
use revm_interpreter::{
    analysis::to_analysed,
    primitives::{Address, Bytecode, Env, LatestSpec, TransactTo},
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

    let caller_address = CALLER_ADDRESS
        .parse::<Address>()
        .expect("could not parse caller address");

    let contract_code = hex::decode(args.contract_code)
        .expect("could not hex decode contract code")
        .into();
    let calldata = hex::decode(args.calldata)
        .expect("could not hex decode calldata")
        .into();

    let mut env = Env::default();
    env.tx.caller = caller_address;
    env.tx.transact_to = TransactTo::create();
    env.tx.data = calldata;

    let bytecode = to_analysed(Bytecode::new_raw(contract_code));
    let bytecode_hash = bytecode.hash_slow();
    let contract = Box::new(Contract::new_env(&env, bytecode, bytecode_hash));

    for _ in 0..args.num_runs {
        let mut interpreter = Interpreter::new(contract.clone(), u64::MAX, false);
        let mut host: DummyHost = DummyHost::new(env.clone());

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
