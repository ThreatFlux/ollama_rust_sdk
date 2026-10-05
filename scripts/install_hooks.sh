#!/bin/sh
# Install only hooks owned by this repository, including in linked worktrees.
set -eu

repo_root=$(git rev-parse --show-toplevel)
hooks_dir=$(git -C "$repo_root" rev-parse --git-path hooks)
case "$hooks_dir" in
    /* | [A-Za-z]:/* | [A-Za-z]:\\*) ;;
    *) hooks_dir="$repo_root/$hooks_dir" ;;
esac
mkdir -p "$hooks_dir"

for name in pre-commit pre-push; do
    source_hook="$repo_root/.githooks/$name"
    installed_hook="$hooks_dir/$name"
    if [ -L "$installed_hook" ]; then
        printf 'Refusing to replace symlink hook: %s\n' "$installed_hook" >&2
        exit 1
    fi
    if [ -e "$installed_hook" ]; then
        if [ ! -f "$installed_hook" ] ||
            ! head -n 2 "$installed_hook" | grep -qx '# ollama_rust_sdk repository hook'; then
            printf 'Refusing to replace foreign hook: %s\n' "$installed_hook" >&2
            exit 1
        fi
        if cmp -s "$source_hook" "$installed_hook"; then
            chmod +x "$installed_hook"
            continue
        fi
    fi
    cp "$source_hook" "$installed_hook"
    chmod +x "$installed_hook"
done
printf 'Installed repository hooks in %s\n' "$hooks_dir"
