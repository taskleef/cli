#!/bin/bash
# Bash completion for todo CLI
# Source this file or add to .bashrc:
#   source /path/to/todo-completion.bash

_todo_get_projects() {
    local api_url="${TODO_API_URL:-https://todo.extroverteddeveloper.com}"
    local api_key="${TODO_API_KEY}"

    if [[ -z "$api_key" ]]; then
        return
    fi

    curl -s -H "X-API-Key: $api_key" "${api_url}/projects" 2>/dev/null | \
        jq -r '.[].title // empty' 2>/dev/null
}

_todo_get_todos() {
    local api_url="${TODO_API_URL:-https://todo.extroverteddeveloper.com}"
    local api_key="${TODO_API_KEY}"

    if [[ -z "$api_key" ]]; then
        return
    fi

    curl -s -H "X-API-Key: $api_key" "${api_url}/todos" 2>/dev/null | \
        jq -r '.[] | select(.isCompleted == false) | .title // empty' 2>/dev/null
}

_todo_completions() {
    local cur prev words cword
    _init_completion || return

    local commands="add list ls inbox show subtask complete done delete rm project help"
    local project_commands="list ls add show delete rm add-todo remove-todo"

    case "${words[1]}" in
        project)
            case "${words[2]}" in
                add-todo|remove-todo)
                    if [[ $cword -eq 3 ]]; then
                        # Complete project names
                        local projects
                        projects=$(_todo_get_projects)
                        if [[ -n "$projects" ]]; then
                            local IFS=$'\n'
                            COMPREPLY=($(compgen -W "$projects" -- "$cur"))
                            # Handle spaces in names
                            if [[ ${#COMPREPLY[@]} -eq 1 && "${COMPREPLY[0]}" == *" "* ]]; then
                                COMPREPLY=("\"${COMPREPLY[0]}\"")
                            fi
                        fi
                    elif [[ $cword -eq 4 ]]; then
                        # Complete todo titles
                        local todos
                        todos=$(_todo_get_todos)
                        if [[ -n "$todos" ]]; then
                            local IFS=$'\n'
                            COMPREPLY=($(compgen -W "$todos" -- "$cur"))
                            if [[ ${#COMPREPLY[@]} -eq 1 && "${COMPREPLY[0]}" == *" "* ]]; then
                                COMPREPLY=("\"${COMPREPLY[0]}\"")
                            fi
                        fi
                    fi
                    ;;
                show|delete|rm)
                    if [[ $cword -eq 3 ]]; then
                        local projects
                        projects=$(_todo_get_projects)
                        if [[ -n "$projects" ]]; then
                            local IFS=$'\n'
                            COMPREPLY=($(compgen -W "$projects" -- "$cur"))
                            if [[ ${#COMPREPLY[@]} -eq 1 && "${COMPREPLY[0]}" == *" "* ]]; then
                                COMPREPLY=("\"${COMPREPLY[0]}\"")
                            fi
                        fi
                    fi
                    ;;
                ""|list|ls|add)
                    if [[ $cword -eq 2 ]]; then
                        COMPREPLY=($(compgen -W "$project_commands" -- "$cur"))
                    fi
                    ;;
                *)
                    if [[ $cword -eq 2 ]]; then
                        COMPREPLY=($(compgen -W "$project_commands" -- "$cur"))
                    fi
                    ;;
            esac
            ;;
        show|complete|done|delete|rm)
            if [[ $cword -eq 2 ]]; then
                local todos
                todos=$(_todo_get_todos)
                if [[ -n "$todos" ]]; then
                    local IFS=$'\n'
                    COMPREPLY=($(compgen -W "$todos" -- "$cur"))
                    if [[ ${#COMPREPLY[@]} -eq 1 && "${COMPREPLY[0]}" == *" "* ]]; then
                        COMPREPLY=("\"${COMPREPLY[0]}\"")
                    fi
                fi
            fi
            ;;
        subtask)
            if [[ $cword -eq 2 ]]; then
                local todos
                todos=$(_todo_get_todos)
                if [[ -n "$todos" ]]; then
                    local IFS=$'\n'
                    COMPREPLY=($(compgen -W "$todos" -- "$cur"))
                    if [[ ${#COMPREPLY[@]} -eq 1 && "${COMPREPLY[0]}" == *" "* ]]; then
                        COMPREPLY=("\"${COMPREPLY[0]}\"")
                    fi
                fi
            fi
            ;;
        *)
            if [[ $cword -eq 1 ]]; then
                COMPREPLY=($(compgen -W "$commands" -- "$cur"))
            fi
            ;;
    esac
}

complete -F _todo_completions todo
