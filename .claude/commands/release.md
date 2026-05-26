# release

Guide through creating a new DINOForge release.

**Usage**: `/release <version>`

**Example**: `/release 0.6.0`

**Arguments**: $ARGUMENTS

## Steps

1. Verify the version format is valid semver (X.Y.Z)
2. Check that all CI tests pass locally first
3. Show what will be in the release (git log since last tag)
4. Update CHANGELOG.md: move [Unreleased] content to [<version>] with today's date
5. Update VERSION file with new version
6. Show the git commands to tag and push (but DO NOT run git push automatically)
7. Remind about the GitHub Actions version-bump workflow alternative

This command guides you through the release process step-by-step without automatic pushing.
