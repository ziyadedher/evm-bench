import { readFile } from "fs/promises";

import { VM } from "@ethereumjs/vm";
import { MAX_INTEGER_BIGINT, toBuffer } from "@ethereumjs/util";
import { Chain, Common, Hardfork } from "@ethereumjs/common";

import { program } from "commander";

const GAS_LIMIT = MAX_INTEGER_BIGINT;

async function main() {
  program
    .option(
      "--contract-code-path <path>",
      "Path to the hex contract code to deploy and run"
    )
    .option(
      "--calldata <hex>",
      "Hex of calldata to use when calling the contract"
    )
    .option("--num-runs <int>", "Number of times to run the benchmark");
  await program.parseAsync();
  
  const contractPath = program.opts().contractCodePath as string;
  const calldata = toBuffer("0x" + program.opts().calldata);
  const numRuns = parseInt(program.opts().numRuns);

  const contractCode = await readFile(contractPath, {
    encoding: "utf-8",
  });
  const contractCodeBytes = toBuffer("0x" + contractCode);

  const vm = await VM.create({
    common: new Common({ chain: Chain.Mainnet, hardfork: Hardfork.London }),
  });

  const createResult = await vm.evm.runCall({
    gasLimit: GAS_LIMIT,
    data: contractCodeBytes,
  });
  if (createResult.execResult.exceptionError) {
    throw createResult.execResult.exceptionError;
  }

  const contractAddress = createResult.createdAddress!;

  for (let i = 0; i < numRuns; i++) {
    const start = performance.now();
    const callResult = await vm.evm.runCall({
      gasLimit: GAS_LIMIT,
      caller: contractAddress,
      origin: contractAddress,
      to: contractAddress,
      data: calldata,
    });
    const end = performance.now();
    if (callResult.execResult.exceptionError) {
      throw callResult.execResult.exceptionError;
    }
    console.log(end - start);
  }
}

main()
  .then(() => process.exit(0))
  .catch((e) => {
    console.error(e);
    process.exit(1);
  });
