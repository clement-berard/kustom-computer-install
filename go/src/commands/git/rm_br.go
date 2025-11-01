package git

import (
	"fmt"
	"os"
	"os/exec"
	"strings"

	"github.com/charmbracelet/bubbles/spinner"
	"github.com/charmbracelet/bubbles/textinput"
	tea "github.com/charmbracelet/bubbletea"
	"github.com/charmbracelet/lipgloss"
	"github.com/spf13/cobra"
)

var (
	dryRun bool
	yes    bool
)

var (
	titleStyle = lipgloss.NewStyle().
			Bold(true).
			Foreground(lipgloss.Color("205"))

	branchStyle = lipgloss.NewStyle().
			Foreground(lipgloss.Color("86"))

	deletedStyle = lipgloss.NewStyle().
			Foreground(lipgloss.Color("42"))

	skippedStyle = lipgloss.NewStyle().
			Foreground(lipgloss.Color("240"))

	dryRunStyle = lipgloss.NewStyle().
			Foreground(lipgloss.Color("214")).
			Bold(true)
)

var RmBrCmd = &cobra.Command{
	Use:   "rm-br",
	Short: "Interactively delete local branches without remote",
	Long:  `Fetch and prune, then interactively delete local branches that don't have a remote counterpart.`,
	Run: func(cmd *cobra.Command, args []string) {
		p := tea.NewProgram(initialModelRmBr())
		if _, err := p.Run(); err != nil {
			fmt.Fprintf(os.Stderr, "Error: %v\n", err)
			os.Exit(1)
		}
	},
}

func init() {
	RmBrCmd.Flags().BoolVar(&dryRun, "dry-run", false, "Show what would be deleted without actually deleting")
	RmBrCmd.Flags().BoolVarP(&yes, "yes", "y", false, "Skip confirmation and delete all branches")
}

type modelRmBr struct {
	spinner         spinner.Model
	textInput       textinput.Model
	stage           string
	branches        []string
	currentBranch   string
	currentBranchIdx int
	waitingConfirm  bool
	deletedBranches []string
	skippedBranches []string
	err             error
	done            bool
}

type fetchDoneMsg struct{ err error }
type branchesFoundMsg struct {
	branches      []string
	currentBranch string
}

func initialModelRmBr() modelRmBr {
	s := spinner.New()
	s.Spinner = spinner.Dot
	s.Style = lipgloss.NewStyle().Foreground(lipgloss.Color("205"))

	ti := textinput.New()
	ti.Placeholder = "y/N"
	ti.CharLimit = 1
	ti.Width = 3

	return modelRmBr{
		spinner:          s,
		textInput:        ti,
		stage:            "fetching",
		currentBranchIdx: 0,
		waitingConfirm:   false,
	}
}

func (m modelRmBr) Init() tea.Cmd {
	return tea.Batch(m.spinner.Tick, fetchAndPrune)
}

func (m modelRmBr) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	switch msg := msg.(type) {
	case tea.KeyMsg:
		if msg.String() == "ctrl+c" {
			m.done = true
			return m, tea.Quit
		}

		if m.waitingConfirm {
			switch msg.String() {
			case "enter":
				answer := strings.ToLower(strings.TrimSpace(m.textInput.Value()))

				if answer == "y" {
					branch := m.branches[m.currentBranchIdx]
					if !dryRun {
						deleteCmd := exec.Command("git", "branch", "-D", branch)
						if err := deleteCmd.Run(); err == nil {
							m.deletedBranches = append(m.deletedBranches, branch)
						}
					} else {
						m.deletedBranches = append(m.deletedBranches, branch)
					}
				} else {
					m.skippedBranches = append(m.skippedBranches, m.branches[m.currentBranchIdx])
				}

				m.currentBranchIdx++
				if m.currentBranchIdx >= len(m.branches) {
					m.done = true
					m.waitingConfirm = false
					return m, tea.Quit
				}

				m.textInput.Reset()
				m.textInput.Focus()
				return m, nil
			default:
				var cmd tea.Cmd
				m.textInput, cmd = m.textInput.Update(msg)
				return m, cmd
			}
		}

	case fetchDoneMsg:
		if msg.err != nil {
			m.err = msg.err
			m.done = true
			return m, tea.Quit
		}
		m.stage = "finding"
		return m, findBranchesToDelete

	case branchesFoundMsg:
		m.branches = msg.branches
		m.currentBranch = msg.currentBranch
		if len(m.branches) == 0 {
			m.done = true
			return m, tea.Quit
		}

		if yes {
			m.stage = "deleting"
			for _, branch := range m.branches {
				if !dryRun {
					deleteCmd := exec.Command("git", "branch", "-D", branch)
					if err := deleteCmd.Run(); err == nil {
						m.deletedBranches = append(m.deletedBranches, branch)
					}
				} else {
					m.deletedBranches = append(m.deletedBranches, branch)
				}
			}
			m.done = true
			return m, tea.Quit
		}

		m.stage = "confirming"
		m.waitingConfirm = true
		m.textInput.Focus()
		return m, textinput.Blink

	default:
		if !m.waitingConfirm {
			var cmd tea.Cmd
			m.spinner, cmd = m.spinner.Update(msg)
			return m, cmd
		}
	}
	return m, nil
}

func (m modelRmBr) View() string {
	if m.done {
		if m.err != nil {
			return titleStyle.Render("❌ Error: ") + m.err.Error() + "\n"
		}
		if len(m.branches) == 0 {
			return titleStyle.Render("✓ No branches to delete") + "\n"
		}

		var result strings.Builder

		if dryRun {
			result.WriteString(dryRunStyle.Render("🔍 DRY RUN - Branches that would be deleted:") + "\n\n")
		} else {
			result.WriteString(titleStyle.Render("✓ Deleted branches:") + "\n\n")
		}

		for _, branch := range m.deletedBranches {
			result.WriteString(deletedStyle.Render("  ✓ "+branch) + "\n")
		}

		if len(m.skippedBranches) > 0 {
			result.WriteString("\n" + skippedStyle.Render("Skipped branches:") + "\n\n")
			for _, branch := range m.skippedBranches {
				result.WriteString(skippedStyle.Render("  - "+branch) + "\n")
			}
		}

		result.WriteString(fmt.Sprintf("\n%s\n", titleStyle.Render(fmt.Sprintf("Total deleted: %d branch(es)", len(m.deletedBranches)))))
		return result.String()
	}

	switch m.stage {
	case "fetching":
		return fmt.Sprintf("%s %s\n", m.spinner.View(), "Fetching and pruning...")
	case "finding":
		return fmt.Sprintf("%s %s\n", m.spinner.View(), "Finding branches without remote...")
	case "deleting":
		return fmt.Sprintf("%s %s\n", m.spinner.View(), "Deleting all branches...")
	case "confirming":
		if m.waitingConfirm && m.currentBranchIdx < len(m.branches) {
			branch := m.branches[m.currentBranchIdx]

			var output strings.Builder
			progress := fmt.Sprintf("(%d/%d) ", m.currentBranchIdx+1, len(m.branches))
			output.WriteString(progress)
			output.WriteString("Delete ")
			output.WriteString(branchStyle.Render("'"+branch+"'"))
			output.WriteString("?")

			if dryRun {
				output.WriteString(" " + dryRunStyle.Render("[DRY RUN]"))
			}

			output.WriteString(" [y/N]: ")
			output.WriteString(m.textInput.View())
			output.WriteString("\n")

			return output.String()
		}
		return ""
	default:
		return ""
	}
}

func fetchAndPrune() tea.Msg {
	cmd := exec.Command("git", "fetch", "--prune", "--quiet")
	err := cmd.Run()
	return fetchDoneMsg{err: err}
}

func findBranchesToDelete() tea.Msg {
	currentBranchCmd := exec.Command("git", "branch", "--show-current")
	currentBranchOutput, err := currentBranchCmd.Output()
	if err != nil {
		return branchesFoundMsg{}
	}
	currentBranch := strings.TrimSpace(string(currentBranchOutput))

	localBranchesCmd := exec.Command("git", "for-each-ref", "--format=%(refname:short)", "refs/heads/")
	localBranchesOutput, err := localBranchesCmd.Output()
	if err != nil {
		return branchesFoundMsg{}
	}

	remoteBranchesCmd := exec.Command("git", "branch", "-r")
	remoteBranchesOutput, err := remoteBranchesCmd.Output()
	if err != nil {
		return branchesFoundMsg{}
	}
	remoteBranches := string(remoteBranchesOutput)

	var branchesToDelete []string
	localBranches := strings.Split(strings.TrimSpace(string(localBranchesOutput)), "\n")

	for _, branch := range localBranches {
		branch = strings.TrimSpace(branch)
		if branch == currentBranch || branch == "" {
			continue
		}
		remoteBranchName := "origin/" + branch
		if !strings.Contains(remoteBranches, remoteBranchName) {
			branchesToDelete = append(branchesToDelete, branch)
		}
	}

	return branchesFoundMsg{
		branches:      branchesToDelete,
		currentBranch: currentBranch,
	}
}
