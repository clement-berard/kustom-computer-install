package main

import (
	"fmt"
	"os"

	"kc-cli/src/commands"

	"github.com/spf13/cobra"
)

var rootCmd = &cobra.Command{
	Use:   "kc-cli",
	Short: "A simple CLI for testing",
	Long:  `A simple CLI created with Cobra to test basic commands.`,
}

func init() {
	rootCmd.AddCommand(commands.RmdCmd)
}

func main() {
	if err := rootCmd.Execute(); err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
}
