use std::{fs, path::PathBuf, str::FromStr, time::Instant};

use akula::{
    execution::{
        address::create_address,
        evm::{
            util::mocked_host::MockedHost, AnalyzedCode, CallKind, InterpreterMessage, StatusCode,
        },
    },
    models::{Address, Revision, U256},
};
use clap::Parser;

/// Akula runner interface
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

    let caller_address = Address::from_str(CALLER_ADDRESS).unwrap();
    let contract_address = create_address(caller_address, 0);

    let contract_code =
        hex::decode(fs::read_to_string(args.contract_code_path).expect("unable to open file"))
            .expect("could not hex decode contract code");
    let calldata = hex::decode(args.calldata).expect("could not hex decode calldata");

    // Set up the EVM with a database and create the contract
    let mut host = MockedHost::default();
    let create_result = AnalyzedCode::analyze(contract_code.as_slice()).execute(
        &mut host,
        &InterpreterMessage {
            kind: CallKind::Call,
            is_static: false,
            depth: 0,
            gas: i64::MAX,
            recipient: contract_address,
            sender: caller_address,
            code_address: contract_address,
            real_sender: caller_address,
            input_data: Default::default(),
            value: U256::ZERO,
        }
        .into(),
        Revision::London,
    );
    match create_result.status_code {
        StatusCode::Success => {}
        reason => panic!("unexpected exit reason while creating: {:?}", reason),
    }

    let call_analyzed = AnalyzedCode::analyze(&create_result.output_data);
    let call_message = InterpreterMessage {
        kind: CallKind::Call,
        is_static: false,
        depth: 0,
        gas: i64::MAX,
        recipient: contract_address,
        sender: caller_address,
        code_address: contract_address,
        real_sender: caller_address,
        input_data: calldata.into(),
        value: U256::ZERO,
    };

    for _ in 0..args.num_runs {
        let timer = Instant::now();
        let call_result = call_analyzed.execute(&mut host, &call_message, Revision::London);
        let dur = timer.elapsed();

        match call_result.status_code {
            StatusCode::Success => {}
            reason => panic!("unexpected exit reason while benchmarking: {:?}", reason),
        }

        println!("{}", dur.as_micros() as f64 / 1e3)
    }
}
