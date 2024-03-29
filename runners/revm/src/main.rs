use clap::Parser;
use revm::{
    interpreter::{
        opcode::make_instruction_table,
        primitives::{address, hex, Bytes, Env, LatestSpec, TransactTo},
        Contract, DummyHost, Interpreter, SharedMemory,
    },
    primitives::{ExecutionResult, Output, ResultAndState},
    Evm,
};
use std::{fs, path::PathBuf, time::Instant};

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

fn main() {
    let args = Args::parse();

    let creation_code: Bytes =
        hex::decode(fs::read_to_string(args.contract_code_path).expect("unable to open file"))
            .expect("could not hex decode contract code")
            .into();
    let calldata: Bytes = hex::decode(args.calldata)
        .expect("could not hex decode calldata")
        .into();

    let caller = address!("1000000000000000000000000000000000000001");

    // Set up the EVM to create the contract.
    let mut create_env = Env::default();
    create_env.tx.caller = caller;
    create_env.tx.transact_to = TransactTo::create();
    create_env.tx.data = creation_code;

    let mut evm = Evm::builder()
        .with_empty_db()
        .with_env(create_env.into())
        .build();
    let ResultAndState { result, state } = evm.transact().expect("EVM failed");
    let ExecutionResult::Success { output, .. } = result else {
        panic!("failed executing bytecode: {result:#?}");
    };
    let Output::Create(_, Some(created_address)) = output else {
        panic!("failed creating contract: {output:#?}");
    };

    // Run the created bytecode with just the interpreter.
    let created_bytecode = state[&created_address]
        .info
        .code
        .as_ref()
        .expect("failed creation");

    let mut run_env = Env::default();
    run_env.tx.caller = caller;
    run_env.tx.transact_to = TransactTo::call(created_address);
    run_env.tx.data = calldata;

    let contract = Contract::new_env(
        &run_env,
        created_bytecode.clone(),
        created_bytecode.hash_slow(),
    );
    let mut host = DummyHost::new(run_env);
    let table = &make_instruction_table::<_, LatestSpec>();

    for _ in 0..args.num_runs {
        let mut interpreter = Interpreter::new(contract.clone().into(), u64::MAX, false);

        let timer = Instant::now();
        let action = interpreter.run(SharedMemory::new(), table, &mut host);
        let dur = timer.elapsed();

        assert!(
            interpreter.instruction_result.is_ok(),
            "interpreter failed with {:?}",
            interpreter.instruction_result
        );
        assert!(action.is_return(), "unexpected exit action: {action:?}");

        host.clear();

        println!("{}", dur.as_micros() as f64 / 1e3)
    }
}
