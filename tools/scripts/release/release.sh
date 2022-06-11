#!/usr/bin/env bash
set -ex

if [[ -z $GITHUB_USERNAME ]]; then 
  echo "Please set your github username"
  exit 1
fi

# Ockam crate bump
gh workflow run create-release-pull-request.yml --ref metaclips/release_automation -R metaclips/ockam
# Sleep for 10 seconds to ensure we are not affected by Github API downtime.
sleep 10
# Wait for workflow run
run_id=$(gh run list --workflow=create-release-pull-request.yml -b metaclips/release_automation -u $GITHUB_USERNAME -L 1 -R metaclips/ockam --json databaseId | jq -r .[0].databaseId)
gh run watch $run_id --exit-status -R metaclips/ockam

read -p "Crate bump pull request created.... Please merge pull request and press enter to start binaries release."
exit 0

# Start release binaries workflow
gh workflow run release-binaries.yml --ref metaclips/release_automation -R metaclips/ockam
# Wait for workflow run
sleep 10
run_id=$(gh run list --workflow=release-binaries.yml -b metaclips/release_automation -u $GITHUB_USERNAME -L 1 -R metaclips/ockam --json databaseId | jq -r .[0].databaseId)
gh run watch $run_id --exit-status -R metaclips/ockam

# Homebrew Release
gh workflow run create-release-pull-request.yml --ref main -R metaclips/homebrew-ockam
# Wait for workflow run
sleep 10
run_id=$(gh run list --workflow=create-release-pull-request.yml -b main -u $GITHUB_USERNAME -L 1 -R metaclips/homebrew-ockam --json databaseId | jq -r .[0].databaseId)
gh run watch $run_id --exit-status -R metaclips/homebrew-ockam

# Terraform Release
gh workflow run create-release.yml --ref main -R metaclips/terraform-provider-ockam
# Wait for workflow run
sleep 10
run_id=$(gh run list --workflow=create-release.yml -b main -u $GITHUB_USERNAME -L 1 -R metaclips/terraform-provider-ockam  --json databaseId | jq -r .[0].databaseId)
gh run watch $run_id --exit-status -R metaclips/terraform-provider-ockam
