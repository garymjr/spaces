#!/bin/bash
# spaces-enter function for proper cd support
spaces-enter() {
    local path
    path=$(spaces enter "$@") || return $?
    cd "$path"
}

# Aliases for convenience
alias se='spaces-enter'
alias sl='spaces list'
alias sc='spaces create'
alias sr='spaces remove'
alias si='spaces info'
alias shk='spaces hook'
