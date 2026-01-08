#!/usr/bin/env bash
# Bash completion for spaces

_spaces() {
    local cur prev words cword
    _init_completion || return

    if [[ $cword -eq 1 ]]; then
        COMPREPLY=($(compgen -W "list create enter remove info hook" -- "$cur"))
        return
    fi

    case ${words[1]} in
        enter|remove|info)
            # Autocomplete worktree names
            local worktrees
            worktrees=$(spaces list 2>/dev/null | awk 'NR>2 {print $1}')
            COMPREPLY=($(compgen -W "$worktrees" -- "$cur"))
            ;;
        create)
            case $prev in
                -b)
                    # Suggest branch names
                    local branches
                    branches=$(git branch -a 2>/dev/null | sed 's/^[* ] //' | sed 's|remotes/origin/||' | sort -u)
                    COMPREPLY=($(compgen -W "$branches" -- "$cur"))
                    ;;
                *)
                    # Suggest -b flag
                    COMPREPLY=($(compgen -W "-b" -- "$cur"))
                    ;;
            esac
            ;;
        hook)
            if [[ $cword -eq 2 ]]; then
                COMPREPLY=($(compgen -W "list $(spaces list 2>/dev/null | awk 'NR>2 {print $1}')" -- "$cur"))
            elif [[ $cword -eq 3 ]]; then
                COMPREPLY=($(compgen -W "pre-create post-create pre-enter post-enter pre-remove post-remove" -- "$cur"))
            fi
            ;;
    esac
}

complete -F _spaces spaces
