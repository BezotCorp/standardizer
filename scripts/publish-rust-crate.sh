#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -ne 2 ]; then
  echo "usage: publish-rust-crate.sh <crate-name> <owner/repository>" >&2
  exit 2
fi

CRATE_NAME="$1"
REPOSITORY="$2"
RELEASE_VERSION="${RELEASE_VERSION:?RELEASE_VERSION is required}"

DECLARED_VERSION="$(
  ./scripts/read-crate-version.sh "$CRATE_NAME"
)"

if [ "$DECLARED_VERSION" != "$RELEASE_VERSION" ]; then
  echo "declared version $DECLARED_VERSION differs from release version $RELEASE_VERSION" >&2
  exit 3
fi

set +e
PUBLICATION_STATE="$(
  ./scripts/check-rust-publication.sh \
    "$CRATE_NAME" \
    "$RELEASE_VERSION" \
    "$REPOSITORY"
)"
PUBLICATION_STATUS=$?
set -e

if [ "$PUBLICATION_STATUS" -gt 1 ]; then
  echo "$PUBLICATION_STATE" >&2
  exit "$PUBLICATION_STATUS"
fi

CRATE_PUBLISHED="$(
  printf '%s\n' "$PUBLICATION_STATE" |
    sed -n 's/^crate_published=//p'
)"

GITHUB_RELEASE_PUBLISHED="$(
  printf '%s\n' "$PUBLICATION_STATE" |
    sed -n 's/^github_release_published=//p'
)"

case "$CRATE_PUBLISHED:$GITHUB_RELEASE_PUBLISHED" in
true:true | true:false | false:true | false:false) ;;
*)
  echo "invalid publication state: $PUBLICATION_STATE" >&2
  exit 3
  ;;
esac

if [ "$CRATE_PUBLISHED" = false ]; then
  for CRATE in ${PUBLISH_CRATES:-$CRATE_NAME}; do
    CRATE_STATUS="$(
      curl \
        --silent \
        --show-error \
        --output /dev/null \
        --write-out '%{http_code}' \
        --header 'User-Agent: standardizer-release-automation (https://github.com/BezotCorp/standardizer)' \
        "https://crates.io/api/v1/crates/$CRATE/$RELEASE_VERSION"
    )"

    case "$CRATE_STATUS" in
    200)
      echo "$CRATE $RELEASE_VERSION is already published on crates.io."
      ;;
    404)
      echo "Publishing $CRATE $RELEASE_VERSION to crates.io..."
      cargo publish \
        --package "$CRATE" \
        --locked
      ;;
    *)
      echo "crates.io returned HTTP $CRATE_STATUS for $CRATE" >&2
      exit 3
      ;;
    esac
  done
else
  echo "$CRATE_NAME $RELEASE_VERSION is already published on crates.io."
fi

if [ "$GITHUB_RELEASE_PUBLISHED" = false ]; then
  if git ls-remote \
    --exit-code \
    --tags \
    origin \
    "refs/tags/$RELEASE_VERSION" \
    >/dev/null 2>&1; then
    echo "Tag $RELEASE_VERSION already exists."
  else
    git tag "$RELEASE_VERSION" origin/main
    git push origin "refs/tags/$RELEASE_VERSION"
  fi

  RELEASE_ASSETS=()
  if [ -n "${RELEASE_ASSETS_DIR:-}" ] && [ -d "$RELEASE_ASSETS_DIR" ]; then
    for ASSET in "$RELEASE_ASSETS_DIR"/*.tar.gz; do
      [ -e "$ASSET" ] && RELEASE_ASSETS+=("$ASSET")
    done
  fi

  ./scripts/read-changelog-release.sh \
    CHANGELOG.md \
    "$RELEASE_VERSION" |
    gh release create "$RELEASE_VERSION" \
      --repo "$REPOSITORY" \
      --title "$RELEASE_VERSION" \
      --notes-file - \
      "${RELEASE_ASSETS[@]}"
else
  echo "GitHub release $RELEASE_VERSION already exists."
fi
