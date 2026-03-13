## Setup
- Board: <your-board-id>
- Columns: Backlog → Breakdown → In Progress → Review → PR Merged → Deployed

**Column IDs** (for mcp__taskleef__card_move):
- Backlog: <column-id>
- Breakdown: <column-id>
- In Progress: <column-id>
- Review: <column-id>
- PR Merged: <column-id>
- Deployed: <column-id>

## Skills

Before any creative work, check if a skill applies:
- write-plan: Before implementation
- using-git-worktrees: Before starting feature work
- finishing-a-development-branch: When work is complete, for cleanup
- systematic-debugging: When tests fail
- code-review: After implementation, before PR

**DO NOT use brainstorming skill** - work directly from card requirements.

## Process

def move_cards(cards):
   for each card in cards:
      move card to "Backlog"

def process_card(card):
   1. **Move card to "In Progress" column**
      - Use: mcp__taskleef__card_move with your In Progress column ID
      - Verify the move succeeded before proceeding

   2. Show card details and study the description for requirements
      - Use: mcp__taskleef__card_list to get full card details

   3. **Requirements analysis**
      - Read card description carefully
      - If requirements are underspecified, add comment to card and move back to Backlog for human input. STOP.

   4. Search codebase first - don't assume not implemented.

   5. **Write plan** (load skill: write-plan)
      - Generate implementation plan
      - Output: .tasks/<card-id>/plan.md
      - Plan informs subtask granularity

   6. If task is large, break into subtasks:
      Even if the card already has sub-tasks, consider if it needs additional.

      if done making subtasks and len(subtasks) > 0:
         let x::xs = subtasks
         move_cards(xs)
         process_card(x)

   7. Create worktree and feature branch (load skill: using-git-worktrees)
      - Use card-id-prefix for worktree name
      - Branch: feature/<human-readable-short-name>

   8. Launch 3 Agents (2 in parallel, 1 sequential)

      Agent 1 (parallel): Backend implementation
      Agent 2 (parallel): Frontend (ClientApp/)

      Each Agent implements using TDD per the plan.
      Each writes to .tasks/<card-id>/{backend,frontend}-done.md when complete.

      WAIT for both agents.

      Agent 3 (sequential): Code Reviewer (load skill: code-review)
      - Review against plan.md requirements
      - Check for missed edge cases from requirements.md
      - Verify FE/BE integration points match
      - Output: .tasks/<card-id>/review.md

      If review fails:
         - Create subtasks for fixes
         - move_cards(fix_subtasks)
         - Loop back to step 7 with fix subtasks

   9. If tests fail (load skill: systematic-debugging):
       - Root cause investigation (don't guess)
       - Hypothesis → test → verify cycle
       - Document in .tasks/<card-id>/debug-log.md
       - Max 3 debug cycles before escalating to human

   10. If tests pass and both frontend and backend builds succeed:
       - Commit changes
       - Push branch
       - Create PR
       - **IMMEDIATELY move card to "Review" column using mcp__taskleef__card_move**

   11. Have a new sub agent run /code-review:code-review on the PR

   12. Address any PR comments

   13. **Merge the PR** using gh pr merge --squash
       - **IMMEDIATELY after merge, move card to "Deployed" column using mcp__taskleef__card_move**
       - **Mark the underlying todo as complete using mcp__taskleef__complete_todo**
       - Verify card is in Deployed column before proceeding

   14. Cleanup (load skill: finishing-a-development-branch):
       - Handles worktree removal and branch cleanup
       - Ensure backend and frontend processes are stopped
       - **VERIFY: Card is in "Deployed" column and todo is marked complete**

## Startup

get cards from "In Progress" column where in "Inbox sub-column"
   |> LIMIT 5
   |> With a new sub-agent (card ->
      Check if there are any open PRs for card. A card in the In Progress inbox at startup means that work has likely been done but interrupted, or the work hasn't been fully completed to the Verification Rules standard

      process_card(card)
   )
   |> parallel

get cards from "Backlog" column where Tags.Contains("Bug")
   |> With a new sub-agent (card -> process_card(card))
   |> parallel


Wait for all sub-agents to finish

**After all agents complete:**
1. Verify all cards have been moved to "Deployed" column
2. Verify all PRs have been merged
3. Verify all todos are marked complete
4. If any cards are still in wrong columns, move them using mcp__taskleef__card_move

If len(alreadyInProgress) + len(backlogCards) = 0 then
   Output <promise>COMPLETE</promise>


## Verification Rules

99999. Majority of verification should be unit and integration tests. Ask yourself: "Do we have sufficient tests to prevent regression?"
999999. Consider: Does this feature require a UI to work properly? Have we implemented that UI?
9999999. UI/frontend changes MUST be verified in Chrome against the local development server before declaring done.
99999999. Assume any unexpected errors are our fault and investigate.
