package commands

import (
	"fmt"
	"os"
	"os/exec"
	"time"

	"github.com/charmbracelet/bubbles/spinner"
	tea "github.com/charmbracelet/bubbletea"
	"github.com/charmbracelet/lipgloss"
	"github.com/spf13/cobra"
)

type model struct {
	spinner  spinner.Model
	quitting bool
	err      error
	done     bool
}

type doneMsg struct{ err error }

func initialModel() model {
	s := spinner.New()
	s.Spinner = spinner.Dot
	s.Style = lipgloss.NewStyle().Foreground(lipgloss.Color("205"))
	return model{spinner: s}
}

func (m model) Init() tea.Cmd {
	return tea.Batch(m.spinner.Tick, removeNodeModules)
}

func (m model) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	switch msg := msg.(type) {
	case tea.KeyMsg:
		m.quitting = true
		return m, tea.Quit
	case doneMsg:
		m.done = true
		m.err = msg.err
		return m, tea.Quit
	default:
		var cmd tea.Cmd
		m.spinner, cmd = m.spinner.Update(msg)
		return m, cmd
	}
}

func (m model) View() string {
	if m.done {
		if m.err != nil {
			return fmt.Sprintf("❌ Error: %v\n", m.err)
		}
		return "✓ All node_modules removed successfully\n"
	}

	cwd, _ := os.Getwd()
	return fmt.Sprintf("%s Remove recursively 'node_modules' in %s\n", m.spinner.View(), cwd)
}

func removeNodeModules() tea.Msg {
	cmd := exec.Command("find", ".", "-name", "node_modules", "-type", "d", "-prune", "-exec", "rm", "-rf", "{}", "+")
	err := cmd.Run()
	time.Sleep(500 * time.Millisecond) // Small delay to see the spinner
	return doneMsg{err: err}
}

var RmdCmd = &cobra.Command{
	Use:   "rmd",
	Short: "Remove all node_modules directories recursively",
	Long:  `Find and remove all node_modules directories recursively from the current directory.`,
	Run: func(cmd *cobra.Command, args []string) {
		p := tea.NewProgram(initialModel())
		if _, err := p.Run(); err != nil {
			fmt.Fprintf(os.Stderr, "Error: %v\n", err)
			os.Exit(1)
		}
	},
}
