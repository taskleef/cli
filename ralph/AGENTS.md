## Kanban Board

- Board: 3bb3ad48-08da-459a-b540-b5b852e41e8b
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

your API key is in ~/.todo.claude.auth to use it pass it to the todo CLI

`todo --auth-file ~/.todo.claude.auth <commands>`

If you ever have a question about syntax use `todo help` to check

## Codebase Notes

The todo CLI command has moved to ~/code/taskleef-cli

We don't use Docker for local development. Instead we have a local instance of postgres 15 running.
We do use Docker to deploy to production
The production deployment code is located in ~/code/infrastructure/envs/production