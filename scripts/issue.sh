#!/usr/bin/env bash
# Keep one GitHub issue per standing problem, found by its label, so a scheduled workflow
# reports a problem once instead of filing a new issue on every run.
#
#   scripts/issue.sh set     <label> <title> <body-file>   open it, or bring the open one up to date
#   scripts/issue.sh report  <label> <title> <body-file>   open it, or add the body as a comment
#   scripts/issue.sh resolve <label> <comment>             close the open one, if any
#
# Needs gh and GH_TOKEN with issues: write. LABEL_COLOR sets a new label's color.
# DRY_RUN=1 reads the repository as usual and prints every write instead of making it.
set -euo pipefail

cmd=${1:?usage: issue.sh set|report|resolve <label> ...}
label=${2:?usage: issue.sh $cmd <label> ...}

write() {
  if [ -n "${DRY_RUN:-}" ]; then
    printf 'would run:'
    printf ' %q' "$@"
    printf '\n'
  else
    "$@"
  fi
}

open_issue=$(gh issue list --label "$label" --state open --limit 1 --json number --jq '.[0].number // empty' 2>/dev/null || true)

case "$cmd" in
  set | report)
    title=${3:?title}
    body=${4:?body file}
    if [ -z "$open_issue" ]; then
      write gh label create "$label" --force --color "${LABEL_COLOR:-fbca04}" \
        --description "Opened and closed by the atlas's scheduled workflows"
      write gh issue create --title "$title" --label "$label" --body-file "$body"
    elif [ "$cmd" = report ]; then
      write gh issue comment "$open_issue" --body-file "$body"
    else
      # Edit only on a change, so the issue's history shows when the problem changed
      current=$(gh issue view "$open_issue" --json title,body --jq '.title + "\n" + .body')
      if [ "$current" != "$title"$'\n'"$(cat "$body")" ]; then
        write gh issue edit "$open_issue" --title "$title" --body-file "$body"
      fi
    fi
    ;;
  resolve)
    comment=${3:?comment}
    if [ -n "$open_issue" ]; then
      write gh issue close "$open_issue" --comment "$comment"
    fi
    ;;
  *)
    echo "issue.sh: unknown command $cmd" >&2
    exit 2
    ;;
esac
