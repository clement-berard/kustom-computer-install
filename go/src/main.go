package main

import (
	"fmt"
	"os"

	"kc-cli/src/commands"
	"kc-cli/src/commands/git"

	"github.com/spf13/cobra"
)

var rootCmd = &cobra.Command{
	Use:   "kc",
	Short: "Kustom CLI",
	Long:  `Kustom CLI commands to improve my daily workflow.`,
}

func init() {
	rootCmd.AddCommand(commands.RmdCmd)
	rootCmd.AddCommand(git.GitCmd)
}

func main() {
	if err := rootCmd.Execute(); err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
}
