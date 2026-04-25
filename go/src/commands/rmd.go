package commands

import (
	"fmt"
	"io/fs"
	"os"
	"path/filepath"
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
	count    int
}

type doneMsg struct {
	err   error
	count int
}

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
		m.count = msg.count
		return m, tea.Quit
	default:
		var cmd tea.Cmd
		m.spinner, cmd = m.spinner.Update(msg)
		return m, cmd
	}
}

func (m model) View() string {
    cwd, _ := os.Getwd()
	if m.done {
		if m.err != nil {
			return fmt.Sprintf("❌ Error: %v\n", m.err)
		}
		if m.count == 0 {
			return fmt.Sprintf("%s NO 'node_modules' in %s\n", m.spinner.View(), cwd)
		}
		return fmt.Sprintf("✓ Removed %d node_modules director%s successfully\n",
			m.count, pluralize(m.count))
	}

	return fmt.Sprintf("%s Remove recursively 'node_modules' in %s\n", m.spinner.View(), cwd)
}

func pluralize(count int) string {
	if count == 1 {
		return "y"
	}
	return "ies"
}

func removeNodeModules() tea.Msg {
	var toRemove []string

	err := filepath.WalkDir(".", func(path string, d fs.DirEntry, err error) error {
		if err != nil {
			// Skip permission errors but continue walking [web:21][web:24]
			if os.IsPermission(err) {
				return nil
			}
			return err
		}

		if d.IsDir() && d.Name() == "node_modules" {
			toRemove = append(toRemove, path)
			return filepath.SkipDir // Don't descend into node_modules [web:19]
		}
		return nil
	})

	if err != nil {
		return doneMsg{err: err, count: 0}
	}

	// Remove all found directories [web:7]
	for _, dir := range toRemove {
		if err := os.RemoveAll(dir); err != nil {
			return doneMsg{err: fmt.Errorf("failed to remove %s: %w", dir, err), count: len(toRemove)}
		}
	}

	time.Sleep(500 * time.Millisecond) // Small delay to see the spinner
	return doneMsg{err: nil, count: len(toRemove)}
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
