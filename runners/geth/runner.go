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
	contractCode string
	calldata     string
	numRuns      int
)

var cmd = &cobra.Command{
	Use:   "runner-geth",
	Short: "go-ethereum runner for evm-bench",
	Run: func(_ *cobra.Command, _ []string) {
		contractCodeBytes := common.Hex2Bytes(contractCode)
		calldataBytes := common.Hex2Bytes(calldata)

		zeroAddress := common.BytesToAddress(common.FromHex("0x0000000000000000000000000000000000000000"))
		callerAddress := common.BytesToAddress(common.FromHex("0x1000000000000000000000000000000000000001"))
		contractAddress := common.BytesToAddress(common.FromHex("0x2000000000000000000000000000000000000002"))

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
		if err != nil {
			fmt.Fprintln(os.Stderr, err)
			os.Exit(1)
		}
		statedb.SetCode(contractAddress, contractCodeBytes)

		zeroValue := big.NewInt(0)
		gasLimit := ^uint64(0)

		msg := types.NewMessage(callerAddress, &contractAddress, 1, zeroValue, gasLimit, zeroValue, zeroValue, zeroValue, calldataBytes, types.AccessList{}, false)
		statedb.PrepareAccessList(callerAddress, &zeroAddress, vm.ActivePrecompiles(rules), msg.AccessList())

		blockContext := core.NewEVMBlockContext(genesis.ToBlock().Header(), nil, &zeroAddress)
		txContext := core.NewEVMTxContext(msg)

		for i := 0; i < numRuns; i++ {
			evm := vm.NewEVM(blockContext, txContext, statedb.Copy(), config, vm.Config{})

			start := time.Now()
			_, _, err := evm.Call(vm.AccountRef(callerAddress), *msg.To(), msg.Data(), msg.Gas(), msg.Value())
			timeTaken := time.Since(start)

			if err != nil {
				fmt.Fprintln(os.Stderr, err)
				os.Exit(1)
			}

			fmt.Println(timeTaken.Microseconds())
		}
	},
}

func init() {
	cmd.Flags().StringVar(&contractCode, "contract-code", "", "Hex of contract code to deploy and run")
	cmd.MarkFlagRequired("contract-code")
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
