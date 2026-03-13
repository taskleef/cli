## Kanban Board

- Board: <your-board-id>
- Backlog column: Backlog
- Working column: In Progress
- Done column: PR Merged

## Build & Test

dotnet build
dotnet test

## Run the servers

Kill anything on port 5090 and 5173 then

`./run.sh` and read the output to determine which port the vue proxy is running on

## TaskLeef CLI

Your API key should be stored in an auth file. Pass it to the todo CLI:

`todo --auth-file ~/.todo.claude.auth <commands>`

If you ever have a question about syntax use `todo help` to check

## Codebase Notes

We don't use Docker for local development. Instead we have a local instance of postgres 15 running.
We do use Docker to deploy to production.
