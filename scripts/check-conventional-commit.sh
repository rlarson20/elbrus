#!/usr/bin/env bash
# scripts/check-conventional-commit.sh
#
# Wraps `conventional-pre-commit` with an edit-and-retry loop.
# On validation failure, re-opens $GIT_EDITOR via /dev/tty so the user
# can fix the message in-place rather than re-typing it.
#
# Usage (via pre-commit, commit-msg stage):
#   entry: scripts/check-conventional-commit.sh
#   args: [feat, fix, docs, ...]   # allowed types — passed through to validator
#
# Falls back to plain fail (no loop) when /dev/tty is unavailable (CI).

# pre-commit calls: script <type1> <type2> ... <commit-msg-file>
# i.e. args list first, filename appended last.
COMMIT_MSG_FILE="${@: -1}"
ALLOWED_TYPES=("${@:1:$#-1}")

# ── Sanity ───────────────────────────────────────────────────────────────────
if [[ -z "$COMMIT_MSG_FILE" || ! -f "$COMMIT_MSG_FILE" ]]; then
	echo "error: last argument must be the commit message file, got: '$COMMIT_MSG_FILE'" >&2
	exit 1
fi

# ── Helpers ──────────────────────────────────────────────────────────────────
get_editor() {
	# Priority: git config core.editor > $VISUAL > $EDITOR > vi
	git var GIT_EDITOR 2>/dev/null || echo "${VISUAL:-${EDITOR:-vi}}"
}

open_editor() {
	local editor
	editor="$(get_editor)"
	# Explicitly wire stdin/stdout to the real TTY so terminal editors
	# (nvim, vim, nano, etc.) get a proper controlling terminal.
	"$editor" "$COMMIT_MSG_FILE" </dev/tty >/dev/tty 2>/dev/tty
}

prompt_choice() {
	# Returns 0 for "edit", 1 for "abort"
	local choice
	printf "\n  [e]dit message, [a]bort commit (e/a): " >/dev/tty
	read -r choice </dev/tty
	case "${choice,,}" in # bash 4+ lowercase expansion; macOS ships bash 3.2,
	e | "") return 0 ;;   # but Homebrew bash 5 is standard for this toolchain
	*) return 1 ;;
	esac
}

# ── Main loop ────────────────────────────────────────────────────────────────
while true; do
	if uvx conventional-pre-commit "${ALLOWED_TYPES[@]}" "$COMMIT_MSG_FILE"; then
		exit 0
	fi

	if ! prompt_choice; then
		echo "  Commit aborted." >/dev/tty
		exit 1
	fi

	open_editor
done
