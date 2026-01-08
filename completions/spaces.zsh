#compdef spaces

_spaces() {
    local -a commands
    commands=(
        'list:List all worktrees'
        'create:Create a new worktree'
        'enter:Enter a worktree (cd)'
        'remove:Remove a worktree'
        'info:Show worktree details'
        'hook:Run a hook manually'
    )

    if (( CURRENT == 2 )); then
        _describe -t commands 'spaces commands' commands
    else
        case $words[2] in
            enter|remove|info)
                # Autocomplete worktree names
                local worktrees
                worktrees=("${(@f)$(spaces list 2>/dev/null | awk 'NR>2 {print $1}')}")
                if [[ -n $worktrees ]]; then
                    _describe -t worktrees 'worktrees' worktrees
                fi
                ;;
            create)
                # Suggest -b flag
                _arguments \
                    '-b[Specify branch]' \
                    '*:worktree name:'
                ;;
            hook)
                if (( CURRENT == 3 )); then
                    _describe -t hooks 'hook commands' '("list:List available hooks")'
                    # Also suggest worktree names
                    local worktrees
                    worktrees=("${(@f)$(spaces list 2>/dev/null | awk 'NR>2 {print $1}')}")
                    if [[ -n $worktrees ]]; then
                        _describe -t worktrees 'worktrees' worktrees
                    fi
                elif (( CURRENT == 4 )); then
                    # Suggest hook events
                    local events=(
                        'pre-create:Run before worktree creation'
                        'post-create:Run after worktree creation'
                        'pre-enter:Run before entering worktree'
                        'post-enter:Run after entering worktree'
                        'pre-remove:Run before worktree removal'
                        'post-remove:Run after worktree removal'
                    )
                    _describe -t events 'hook events' events
                fi
                ;;
        esac
    fi
}

_spaces "$@"
