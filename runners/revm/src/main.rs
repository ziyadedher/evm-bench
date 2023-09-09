use std::{str::FromStr, time::Instant};

use bytes::Bytes;
use clap::Parser;
use revm_interpreter::{
    analysis::to_analysed,
    primitives::{Bytecode, Env, LatestSpec, TransactTo, B160},
    Contract, DummyHost, InstructionResult, Interpreter,
};

extern crate alloc;

// 608060405234801561000f575f80fd5b5060043610610029575f3560e01c806330627b7c1461002d575b5f80fd5b610035610037565b005b5f5b614e2081101561007e578060405160200161005491906100aa565b60405160208183030381529060405280519060200120508080610076906100f1565b915050610039565b50565b5f819050919050565b5f819050919050565b6100a461009f82610081565b61008a565b82525050565b5f6100b58284610093565b60208201915081905092915050565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52601160045260245ffd5b5f6100fb82610081565b91507fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff820361012d5761012c6100c4565b5b60018201905091905056fea2646970667358221220d8fd009a81c446acc64d9f0f213de571e93b1befa4922b22514132e65dea4b4664736f6c63430008150033
// 608060405234801561001057600080fd5b506004361061002b5760003560e01c806330627b7c14610030575b600080fd5b61003861003a565b005b60005b614e20811015610082578060405160200161005891906100b0565b6040516020818303038152906040528051906020012050808061007a906100fa565b91505061003d565b50565b6000819050919050565b6000819050919050565b6100aa6100a582610085565b61008f565b82525050565b60006100bc8284610099565b60208201915081905092915050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052601160045260246000fd5b600061010582610085565b91507fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff8203610137576101366100cb565b5b60018201905091905056fea26469706673582212201b7a4dd5500502af432ee7c1a1dbbda9705ff4ddcb961cbddd255d0c11e9dac664736f6c63430008120033

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

    for _ in 0..args.num_runs {
        let mut env = Env::default();
        env.tx.caller = caller_address;
        env.tx.transact_to = TransactTo::create();
        env.tx.data = calldata.clone();

        let bytecode = to_analysed(Bytecode::new_raw(contract_code.clone()));
        let bytecode_hash = bytecode.hash_slow();
        let contract = Box::new(Contract::new_env(&env, bytecode, bytecode_hash));

        let mut interpreter = Interpreter::new(contract.clone(), u64::MAX, false);
        let mut host: DummyHost = DummyHost::new(env);

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
