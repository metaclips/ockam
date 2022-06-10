#!/usr/bin/env bash
set -ex

if [[ -z $GITHUB_USERNAME ]]; then 
  echo "Please set your github username"
  exit 1
fi

owner="metaclips"

# Ockam crate bump
gh workflow run create-release-pull-request.yml --ref metaclips/release_automation -R $owner/ockam
# Sleep for 10 seconds to ensure we are not affected by Github API downtime.
sleep 10
# Wait for workflow run
run_id=$(gh run list --workflow=create-release-pull-request.yml -b metaclips/release_automation -u $GITHUB_USERNAME -L 1 -R $owner/ockam --json databaseId | jq -r .[0].databaseId)
gh run watch $run_id --exit-status -R $owner/ockam

read -p "Crate bump pull request created.... Please merge pull request and press enter to start binaries release."
read -p "Script requires draft release has been published and tag created to accurately use latest tag.... Press enter if draft release has been published."

# Start release binaries workflow
gh workflow run release-binaries.yml --ref metaclips/release_automation --field tag= -R $owner/ockam
# Wait for workflow run
sleep 10
run_id=$(gh run list --workflow=release-binaries.yml -b metaclips/release_automation -u $GITHUB_USERNAME -L 1 -R $owner/ockam --json databaseId | jq -r .[0].databaseId)
gh run watch $run_id --exit-status -R $owner/ockam

read -p "Draft release has been created, please vet and release then press enter to start homebrew and terraform CI"

# Get latest tag
latest_tag_name=$(curl -H "Accept: application/vnd.github.v3+json" https://api.github.com/repos/${owner}/ockam/releases/latest | jq -r .tag_name)

# Homebrew Release
gh workflow run create-release-pull-request.yml --ref main -R $owner/homebrew-ockam -F tag=$latest_tag_name
# Wait for workflow run
sleep 10
run_id=$(gh run list --workflow=create-release-pull-request.yml -b main -u $GITHUB_USERNAME -L 1 -R $owner/homebrew-ockam --json databaseId | jq -r .[0].databaseId)
gh run watch $run_id --exit-status -R $owner/homebrew-ockam

# Terraform Release
gh workflow run create-release-pull-request.yml --ref main -R $owner/terraform-provider-ockam -F tag=$latest_tag_name
# Wait for workflow run
sleep 10
run_id=$(gh run list --workflow=create-release-pull-request.yml -b main -u $GITHUB_USERNAME -L 1 -R $owner/terraform-provider-ockam  --json databaseId | jq -r .[0].databaseId)
gh run watch $run_id --exit-status -R $owner/terraform-provider-ockam

read -p "Terraform draft release has been created, please vet and release then press enter to start Terraform binary release"
# GITHUB_USERNAME=metaclips ./tools/scripts/release/release.sh

gh workflow run release.yml --ref main -R $owner/terraform-provider-ockam -F tag=$latest_tag_name
# Wait for workflow run
sleep 10
run_id=$(gh run list --workflow=release.yml -b main -u $GITHUB_USERNAME -L 1 -R $owner/terraform-provider-ockam  --json databaseId | jq -r .[0].databaseId)
gh run watch $run_id --exit-status -R $owner/terraform-provider-ockam
