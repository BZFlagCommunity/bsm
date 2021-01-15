#!/bin/bash

_bsm_completions(){
  local prev="${COMP_WORDS[COMP_CWORD-1]}"

  case "${prev}" in
    status|start|stop|reports|enable|disable)
      local base=./maps/
      _get_comp_words_by_ref cur;
      cur="$base$cur"
      _filedir
      COMPREPLY=("${COMPREPLY[@]#$base}")
      return 0
      ;;
    *)
    ;;
  esac

  if [[ ${COMP_CWORD} -le 1 ]]; then
    COMPREPLY=($(compgen -W "help status list start stop reports enable disable" -- "${COMP_WORDS[1]}"))
  fi
}

complete -F _bsm_completions bsm
