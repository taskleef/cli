## Setup
- Board: 3bb3ad48
- Backlog column: Backlog
- Working column: In Progress
- Review column: Review
- Done column: PR Merged

## Skills

You have access to Superpowers skills. Before any creative work, 
check if a skill applies:

- brainstorming: Before breaking down cards
- write-plan: Before implementation
- systematic-debugging: When tests fail
- code-review: After implementation, before PR

Search for skills: find-skills <query>
Use a skill: Read the SKILL.md and follow its process.

Skills location: ~/.config/superpowers/skills/

## Process

def move_cards(cards):
   for each card in cards:
      todo board 3bb3ad48 move <card-id> "Backlog"

def process_card(card):
   1. Assign and move to "Breakdown":
      todo board 3bb3ad48 assign <card-id>
      todo board 3bb3ad48 move <card-id> "Breakdown"

   2. Get card details:
      todo show <card-id>
      Study the description for requirements.

   3. **NEW: Brainstorm phase**
      Load skill: brainstorming
      - Clarify ambiguous requirements
      - Identify edge cases
      - Surface unknowns ("what happens if X?")
      - Output: .tasks/<card-id>/requirements.md
      
      If requirements are underspecified, add comments to card `todo comment <card-id> "message"` and move back to Backlog for human input.  STOP.

   4. Search codebase first - don't assume not implemented.

   5. **NEW: Write plan before subtask creation**
      Load skill: write-plan
      - Generate implementation plan from requirements.md
      - Output: .tasks/<card-id>/plan.md
      - Plan informs subtask granularity

   6. If task is large, break into subtasks:
      Even if the card already has sub-tasks, consider if it needs additional

      let parent = <card-id>
      todo subtask <card-id> "Subtask description"

      if done making subtasks and len(subtasks) > 0: 
         let x::xs = subtasks
         move_cards(xs)
         process_card(x)
   
   6. todo board 3bb3ad48 move <card-id> "In Progress"

   7. Create worktree and feature branch:
      git worktree add ../workspaces/<card-id-prefix> -b feature/<human-readable-short-name>
      cd ../workspaces/<card-id-prefix>

   8. Launch 3 Agents (2 in parallel , 1 sequential)

      Agent 1 (parallel): Backend implementation
      Agent 2 (parallel): Frontend (ClientApp/)

      Each Agent implements using TDD per the plan.
      Each writes to .tasks/<card-id>/{backend,frontend}-done.md when complete.

      WAIT for both agents.

      Agent 3 (sequential): Code Reviewer
      Load skill: code-review
      - Review against plan.md requirements
      - Check for missed edge cases from requirements.md
      - Verify FE/BE integration points match
      - Output: .tasks/<card-id>/review.md
      
      If review fails:
         - Create subtasks for fixes
         - move_cards(fix_subtasks)
         - Loop back to step 7 with fix subtasks

   8. If tests fail:
      Load skill: systematic-debugging
      - Root cause investigation (don't guess)
      - Hypothesis → test → verify cycle
      - Document in .tasks/<card-id>/debug-log.md
      - Max 3 debug cycles before escalating to human

   9. If tests pass and both frontend and backend builds succeeds:
      - Commit changes
      - Push branch: git push -u origin feature/<card-id-prefix>
      - Create PR: gh pr create --fill

   10. Have a new sub agent run /code-review:code-review on the PR

   11. Address any PR comments

   12. todo board 3bb3ad48 done <card-id>

   13. Cleanup:
      cd ../main-repo
      git worktree remove ../workspaces/<card-id-prefix>
      git branch -d feature/<card-id-prefix>
      ensure that any backend processes are not running
      ensure that any frontend vue proxy processes not running

let alreadyInProgressCards = `todo board 3bb3ad48 column "In Progress"` where in "Inbox" sub-column
let card::_ = alreadyInProgressCards

Check if there are any open PRs for card. A card in the In Progress inbox at startup means that work has likely been done but interrupted, or the work hasn't been fully completed to the Verification Rules standard

process_card(card)

let backlogCards = `todo board 3bb3ad48 column Backlog`
let card::_ = backlogCards
process_card(card)

If len(alreadyInProgress) + len(backlogCards) = 0 then Output <promise>COMPLETE</promise>


## Verification Rules

99999. Majority of verification should be unit and integration tests. Ask yourself: "Do we have sufficient tests to prevent regression?"
999999. Consider: Does this feature require a UI to work properly? Have we implemented that UI?
9999999. UI/frontend changes MUST be verified in Chrome against the local development server before declaring done. 
99999999. Assume any unexpected errors are our fault and investigate.
