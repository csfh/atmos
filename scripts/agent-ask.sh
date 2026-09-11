#!/bin/bash
# Ask the machine's default coding agent one question, without a terminal.
#
# omarchy-agent launches an interactive TUI, which is the right thing for a
# keybinding and the wrong thing here: Atmos needs an answer back, in the
# window, with nothing opening over what the user was doing. Every one of
# these agents has a non-interactive mode, so this maps the configured
# default onto its headless form and prints the reply on stdout.
#
# The prompt arrives on stdin, never as an argument. A user's description can
# contain anything at all and argv is not the place for it.
set -uo pipefail

prompt=$(cat)
[[ -z ${prompt//[[:space:]]/} ]] && { echo "agent-ask: empty prompt" >&2; exit 2; }

agent=${ATMOS_AGENT:-}
[[ -z $agent ]] && command -v omarchy-default-agent >/dev/null 2>&1 && agent=$(omarchy-default-agent 2>/dev/null || true)

if [[ -z $agent ]]; then
  echo "agent-ask: no default agent is set" >&2
  exit 3
fi

if ! command -v "$agent" >/dev/null 2>&1; then
  echo "agent-ask: $agent is the default agent but is not installed" >&2
  exit 4
fi

# Non-interactive invocation per agent. --skip-git-repo-check and the various
# auto-approve flags matter because a prompt waiting for a keypress with no
# terminal attached is a hang, not an error, and would look to the user like
# Atmos had frozen.
case "$agent" in
  codex)
    exec codex exec --skip-git-repo-check "$prompt"
    ;;
  claude)
    exec claude -p "$prompt"
    ;;
  grok)
    exec grok --single "$prompt"
    ;;
  gemini)
    exec gemini -p "$prompt"
    ;;
  opencode)
    exec opencode run "$prompt"
    ;;
  crush)
    exec crush run "$prompt"
    ;;
  copilot)
    exec copilot -p "$prompt"
    ;;
  *)
    echo "agent-ask: $agent has no known non-interactive mode" >&2
    exit 5
    ;;
esac
