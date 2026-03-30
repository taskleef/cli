#compdef todo
# Zsh completion for todo CLI
# Install by adding to fpath or sourcing:
#   source /path/to/todo-completion.zsh

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

_todo() {
    local -a commands project_commands
    commands=(
        'add:Create a new todo'
        'list:List all todos'
        'ls:List all todos'
        'inbox:List todos in inbox'
        'show:Show a todo with its subtasks'
        'subtask:Add a subtask to a todo'
        'complete:Mark a todo as complete'
        'done:Mark a todo as complete'
        'delete:Delete a todo'
        'rm:Delete a todo'
        'project:Project management commands'
        'help:Show help'
    )

    project_commands=(
        'list:List all projects'
        'ls:List all projects'
        'add:Create a new project'
        'show:Show project with its todos'
        'delete:Delete a project'
        'rm:Delete a project'
        'add-todo:Add a todo to a project'
        'remove-todo:Remove a todo from a project'
    )

    if (( CURRENT == 2 )); then
        _describe -t commands 'todo commands' commands
        return
    fi

    case "${words[2]}" in
        project)
            if (( CURRENT == 3 )); then
                _describe -t project-commands 'project commands' project_commands
                return
            fi
            case "${words[3]}" in
                add-todo|remove-todo)
                    if (( CURRENT == 4 )); then
                        local -a projects
                        projects=(${(f)"$(_todo_get_projects)"})
                        if [[ -n "$projects" ]]; then
                            _values 'project' $projects
                        fi
                    elif (( CURRENT == 5 )); then
                        local -a todos
                        todos=(${(f)"$(_todo_get_todos)"})
                        if [[ -n "$todos" ]]; then
                            _values 'todo' $todos
                        fi
                    fi
                    ;;
                show|delete|rm)
                    if (( CURRENT == 4 )); then
                        local -a projects
                        projects=(${(f)"$(_todo_get_projects)"})
                        if [[ -n "$projects" ]]; then
                            _values 'project' $projects
                        fi
                    fi
                    ;;
            esac
            ;;
        show|complete|done|delete|rm)
            if (( CURRENT == 3 )); then
                local -a todos
                todos=(${(f)"$(_todo_get_todos)"})
                if [[ -n "$todos" ]]; then
                    _values 'todo' $todos
                fi
            fi
            ;;
        subtask)
            if (( CURRENT == 3 )); then
                local -a todos
                todos=(${(f)"$(_todo_get_todos)"})
                if [[ -n "$todos" ]]; then
                    _values 'parent todo' $todos
                fi
            fi
            ;;
    esac
}

_todo "$@"
