# Taskleef Project

## Directory Structure
This directory contains multiple git worktrees sharing a common repository:
- `main/` - Main development branch (default)
- Other directories are git worktrees for parallel work

## Working Directory
Unless a specific worktree is mentioned, work in `main/`.
Run git and build commands from the appropriate worktree directory.

## Permissions
The `.claude/` directory here contains shared permissions for all worktrees.

## Workflow
When you think you understand a new task, before you begin work on it create a branch and git worktree for the work


## Related Projects
Related projects live as sibling directories (independent git repos):
- `taskleef-cli/` - Command-line interface (public repo)
