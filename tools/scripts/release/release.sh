#!/usr/bin/env bash
set -ex

if [[ -z $GITHUB_USERNAME ]]; then 
  echo "Please set your github username"
  exit 1
fi

function ockam_bump() {
  gh workflow run create-release-pull-request.yml --ref metaclips/release_automation\
    -F git_tag="$GIT_TAG" -F modified_release="$BUMP_MODIFIED_RELEASE"\
    -F release_version="$RELEASE_VERSION" -F bumped_dep_crates_version="$BUMPED_DEP_CRATES_VERSION"\
    -R $owner/ockam

  # Sleep for 10 seconds to ensure we are not affected by Github API downtime.
  sleep 10
  # Wait for workflow run
  run_id=$(gh run list --workflow=create-release-pull-request.yml -b metaclips/release_automation -u $GITHUB_USERNAME -L 1 -R $owner/ockam --json databaseId | jq -r .[0].databaseId)
  gh run watch $run_id --exit-status -R $owner/ockam
}

function ockam_crate_release() {
  gh workflow run publish_crates.yml --ref metaclips/release_automation \
    -F git_tag="$GIT_TAG" -F exclude_crates="$EXCLUDE_CRATES" \
    -F recent_failure="$RECENT_FAILURE" -R $owner/ockam
  # Sleep for 10 seconds to ensure we are not affected by Github API downtime.
  sleep 10
  # Wait for workflow run
  run_id=$(gh run list --workflow=publish_crates.yml -b metaclips/release_automation -u $GITHUB_USERNAME -L 1 -R $owner/ockam --json databaseId | jq -r .[0].databaseId)
  gh run watch $run_id --exit-status -R $owner/ockam
}

function release_ockam_binaries() {
  gh workflow run release-binaries.yml --ref metaclips/release_automation -F git_tag="$GIT_TAG" -R $owner/ockam
  # Wait for workflow run
  sleep 10
  run_id=$(gh run list --workflow=release-binaries.yml -b metaclips/release_automation -u $GITHUB_USERNAME -L 1 -R $owner/ockam --json databaseId | jq -r .[0].databaseId)
  gh run watch $run_id --exit-status -R $owner/ockam
}

function homebrew_repo_bump() {
  gh workflow run create-release-pull-request.yml --ref main -R $owner/homebrew-ockam -F tag=$1
  # Wait for workflow run
  sleep 10
  run_id=$(gh run list --workflow=create-release-pull-request.yml -b main -u $GITHUB_USERNAME -L 1 -R $owner/homebrew-ockam --json databaseId | jq -r .[0].databaseId)
  gh run watch $run_id --exit-status -R $owner/homebrew-ockam
}

function terraform_repo_bump() {
  gh workflow run create-release-pull-request.yml --ref main -R $owner/terraform-provider-ockam -F tag=$1
  # Wait for workflow run
  sleep 10
  run_id=$(gh run list --workflow=create-release-pull-request.yml -b main -u $GITHUB_USERNAME -L 1 -R $owner/terraform-provider-ockam  --json databaseId | jq -r .[0].databaseId)
  gh run watch $run_id --exit-status -R $owner/terraform-provider-ockam
}

function terraform_binaries_release() {
  gh workflow run release.yml --ref main -R $owner/terraform-provider-ockam -F tag=$1
  # Wait for workflow run
  sleep 10
  run_id=$(gh run list --workflow=release.yml -b main -u $GITHUB_USERNAME -L 1 -R $owner/terraform-provider-ockam  --json databaseId | jq -r .[0].databaseId)
  gh run watch $run_id --exit-status -R $owner/terraform-provider-ockam
}

owner="metaclips"

if [[ -z $SKIP_OCKAM_BUMP || $SKIP_OCKAM_BUMP == false ]]; then
  ockam_bump
  read -p "Crate bump pull request created.... Please merge pull request and press enter to start binaries release."
fi

if [[ -z $SKIP_OCKAM_BINARY_RELEASE || $SKIP_OCKAM_BINARY_RELEASE == false ]]; then
  release_ockam_binaries
  read -p "Draft release has been created, please vet and release then press enter to start homebrew and terraform CI"
  read -p "Script requires draft release to be published and tag created to accurately use latest tag.... Press enter if draft release has been published."
fi

# Get latest tag
if [[ -z $LATEST_TAG_NAME ]]; then
  latest_tag_name=$(curl -H "Accept: application/vnd.github.v3+json" https://api.github.com/repos/${owner}/ockam/releases/latest | jq -r .tag_name)
  read -p "Latest tag is $latest_tag_name press enter is correct"
else
  latest_tag_name=$LATEST_TAG_NAME
fi

# Homebrew Release
if [[ -z $SKIP_HOMEBREW_BUMP || $SKIP_HOMEBREW_BUMP == false ]]; then
  homebrew_repo_bump $latest_tag_name
  echo "Homebrew bump successful"
fi

if [[ -z $SKIP_TERRAFORM_BUMP || $SKIP_TERRAFORM_BUMP == false ]]; then
  terraform_repo_bump $latest_tag_name
fi

read -p "Terraform draft release has been created, please vet and release then press enter to start Terraform binary release"

if [[ -z $SKIP_TERRAFORM_BINARY_RELEASE || $SKIP_TERRAFORM_BINARY_RELEASE == false ]]; then
  terraform_binaries_release $latest_tag_name
fi

echo "Release Done 🚀🚀🚀"
# GITHUB_USERNAME=metaclips ./tools/scripts/release/release.sh