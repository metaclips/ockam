#!/usr/bin/env bash
set -ex

if [[ -z $GITHUB_USERNAME ]]; then 
  echo "Please set your github username"
  exit 1
fi

# Ockam Binary Release
#
# Run bump CI
gh workflow run create-release-pull-request.yml --ref metaclips/release_automation -R metaclips/ockam

# Sleep for 10 seconds to ensure we are not affected by Github API downtime.
sleep 10

# TODO Show workflow run
run_id=$(gh run list --workflow=create-release-pull-request.yml -b metaclips/release_automation -u $GITHUB_USERNAME -L 1 -R metaclips/ockam --json databaseId | jq -r .[0].databaseId)
echo $run_id
gh run watch $run_id --exit-status -R metaclips/ockam

read -p "Crate bump pull request created.... Press enter to start binaries release."
exit 0

# Release workflow
gh workflow run release.yml --ref develop -R build-trust/ockam

# Wait for workflow run
gh workflow list 
gh run watch


# Homebrew Release
#
# Create PR
gh workflow run create-release-pull-request.yml --ref main
# Show workflow run

# Terraform Release
gh workflow run release.yml --ref main
# Show workflow run
