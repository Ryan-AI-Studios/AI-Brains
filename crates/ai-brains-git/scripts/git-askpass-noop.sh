#!/bin/sh
# No-op GIT_ASKPASS helper for non-interactive automation.
# Git invokes this as: askpass "Password for ..."
# Exit 0 immediately with empty stdout so credential prompts fail closed
# without hanging on a TTY/GUI prompt.
exit 0
