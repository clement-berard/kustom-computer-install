package git

import (
	"github.com/spf13/cobra"
)

var GitCmd = &cobra.Command{
	Use:   "git",
	Short: "Git related commands",
	Long:  `Collection of git utility commands`,
}

func init() {
	GitCmd.AddCommand(RmBrCmd)
}
