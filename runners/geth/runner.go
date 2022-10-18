package main

import (
	"fmt"
	"math/big"
	"os"
	"time"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/core"
	"github.com/ethereum/go-ethereum/core/rawdb"
	"github.com/ethereum/go-ethereum/core/state"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/core/vm"
	"github.com/ethereum/go-ethereum/params"
	"github.com/spf13/cobra"
)

var (
	contractCodePath string
	calldata         string
	numRuns          int
)

func check(e error) {
	if e != nil {
		fmt.Fprintln(os.Stderr, e)
		os.Exit(1)
	}
}

var cmd = &cobra.Command{
	Use:   "runner-geth",
	Short: "go-ethereum runner for evm-bench",
	Run: func(_ *cobra.Command, _ []string) {
		contractCodeHex, err := os.ReadFile(contractCodePath)
		check(err)

		contractCodeBytes := common.Hex2Bytes(string(contractCodeHex))
		calldataBytes := common.Hex2Bytes(calldata)

		zeroAddress := common.BytesToAddress(common.FromHex("0x0000000000000000000000000000000000000000"))
		callerAddress := common.BytesToAddress(common.FromHex("0x1000000000000000000000000000000000000001"))

		config := params.MainnetChainConfig
		rules := config.Rules(config.LondonBlock, false)
		defaultGenesis := core.DefaultGenesisBlock()
		genesis := &core.Genesis{
			Config:     config,
			Coinbase:   defaultGenesis.Coinbase,
			Difficulty: defaultGenesis.Difficulty,
			GasLimit:   defaultGenesis.GasLimit,
			Number:     config.LondonBlock.Uint64(),
			Timestamp:  defaultGenesis.Timestamp,
			Alloc:      defaultGenesis.Alloc,
		}

		statedb, err := state.New(common.Hash{}, state.NewDatabase(rawdb.NewMemoryDatabase()), nil)
		check(err)

		zeroValue := big.NewInt(0)
		gasLimit := ^uint64(0)

		createMsg := types.NewMessage(callerAddress, &zeroAddress, 0, zeroValue, gasLimit, zeroValue, zeroValue, zeroValue, contractCodeBytes, types.AccessList{}, false)
		statedb.PrepareAccessList(callerAddress, &zeroAddress, vm.ActivePrecompiles(rules), createMsg.AccessList())

		blockContext := core.NewEVMBlockContext(genesis.ToBlock().Header(), nil, &zeroAddress)
		txContext := core.NewEVMTxContext(createMsg)
		evm := vm.NewEVM(blockContext, txContext, statedb, config, vm.Config{})
		_, contractAddress, _, err := evm.Create(vm.AccountRef(callerAddress), contractCodeBytes, gasLimit, new(big.Int))
		check(err)

		msg := types.NewMessage(callerAddress, &contractAddress, 1, zeroValue, gasLimit, zeroValue, zeroValue, zeroValue, calldataBytes, types.AccessList{}, false)
		for i := 0; i < numRuns; i++ {
			snapshot := statedb.Snapshot()
			statedb.PrepareAccessList(msg.From(), msg.To(), vm.ActivePrecompiles(rules), msg.AccessList())

			start := time.Now()
			_, _, err := evm.Call(vm.AccountRef(callerAddress), *msg.To(), msg.Data(), msg.Gas(), msg.Value())
			timeTaken := time.Since(start)

			fmt.Println(float64(timeTaken.Microseconds()) / 1e3)

			check(err)

			statedb.RevertToSnapshot(snapshot)
		}
	},
}

func init() {
	cmd.Flags().StringVar(&contractCodePath, "contract-code-path", "", "Path to the hex contract code to deploy and run")
	cmd.MarkFlagRequired("contract-code-path")
	cmd.Flags().StringVar(&calldata, "calldata", "", "Hex of calldata to use when calling the contract")
	cmd.MarkFlagRequired("calldata")
	cmd.Flags().IntVar(&numRuns, "num-runs", 0, "Number of times to run the benchmark")
	cmd.MarkFlagRequired("num-runs")
}

func main() {
	if err := cmd.Execute(); err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
}
