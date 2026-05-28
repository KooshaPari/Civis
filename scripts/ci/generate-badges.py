#!/usr/bin/env python3
"""
Generate CI status badges for README.md from GitHub workflows.

This script queries the GitHub API to fetch the latest run status for key workflows
and generates markdown badge entries for display in README.md.

Usage:
  python generate-badges.py --repo KooshaPari/Dino --token <GITHUB_TOKEN>
  python generate-badges.py --help
"""

import argparse
import json
import os
import sys
import subprocess
from datetime import datetime
from typing import Optional, Dict, List
from urllib.request import urlopen, Request
from urllib.error import URLError, HTTPError


class GitHubWorkflowClient:
    """Client for GitHub API workflow operations."""

    def __init__(self, repo: str, token: Optional[str] = None):
        self.repo = repo
        self.token = token or os.environ.get("GITHUB_TOKEN", "")
        self.api_base = "https://api.github.com/repos"

    def get_latest_workflow_runs(self, workflow: str, limit: int = 5) -> List[Dict]:
        """Fetch latest runs for a given workflow file."""
        url = f"{self.api_base}/{self.repo}/actions/workflows/{workflow}/runs?per_page={limit}"
        headers = {"Accept": "application/vnd.github.v3+json"}
        if self.token:
            headers["Authorization"] = f"token {self.token}"

        try:
            req = Request(url, headers=headers)
            with urlopen(req) as response:
                data = json.loads(response.read().decode())
                return data.get("workflow_runs", [])
        except (URLError, HTTPError) as e:
            print(f"Error fetching workflow runs for {workflow}: {e}", file=sys.stderr)
            return []

    def get_workflow_status(self, workflow: str) -> Optional[Dict]:
        """Get the latest status for a workflow."""
        runs = self.get_latest_workflow_runs(workflow, limit=1)
        if not runs:
            return None

        run = runs[0]
        return {
            "name": run.get("name", workflow),
            "status": run.get("status", "unknown"),
            "conclusion": run.get("conclusion", ""),
            "updated_at": run.get("updated_at", ""),
            "html_url": run.get("html_url", ""),
        }


def status_badge(workflow_name: str, status: str, conclusion: Optional[str] = "") -> str:
    """Generate a markdown badge for a workflow status."""
    if status == "completed":
        if conclusion == "success":
            color = "brightgreen"
            label = "passing"
        elif conclusion == "failure":
            color = "red"
            label = "failing"
        elif conclusion == "cancelled":
            color = "yellow"
            label = "cancelled"
        else:
            color = "lightgrey"
            label = "unknown"
    elif status == "in_progress":
        color = "blue"
        label = "running"
    else:
        color = "lightgrey"
        label = "queued"

    return (
        f"[![{workflow_name}](https://img.shields.io/badge/{workflow_name}-{label}-{color})"
        f"](https://github.com/{args.repo}/actions/workflows/{workflow_name}.yml)"
    )


def generate_badge_section(client: GitHubWorkflowClient, workflows: List[str]) -> str:
    """Generate a markdown section with CI status badges."""
    badges = []
    for workflow in workflows:
        status_info = client.get_workflow_status(workflow)
        if status_info:
            badge = status_badge(
                workflow,
                status_info["status"],
                status_info.get("conclusion", ""),
            )
            badges.append(badge)

    return "\n".join(badges) if badges else "No workflows found."


def update_readme(badge_section: str, repo: str) -> None:
    """Update README.md with CI status badges."""
    readme_path = "README.md"

    if not os.path.exists(readme_path):
        print(f"Warning: {readme_path} not found", file=sys.stderr)
        return

    with open(readme_path, "r") as f:
        content = f.read()

    # Find or create CI Status section
    ci_marker_start = "<!-- CI_STATUS_START -->"
    ci_marker_end = "<!-- CI_STATUS_END -->"

    if ci_marker_start in content:
        # Replace existing section
        start_idx = content.find(ci_marker_start)
        end_idx = content.find(ci_marker_end)
        if end_idx != -1:
            new_content = (
                content[:start_idx]
                + ci_marker_start
                + "\n\n"
                + badge_section
                + "\n\n"
                + content[end_idx:]
            )
        else:
            print("Error: CI_STATUS_END marker not found", file=sys.stderr)
            return
    else:
        # Append new section after the first H1
        h1_end = content.find("\n", content.find("#"))
        if h1_end != -1:
            new_content = (
                content[: h1_end + 1]
                + "\n"
                + ci_marker_start
                + "\n\n"
                + badge_section
                + "\n\n"
                + ci_marker_end
                + "\n"
                + content[h1_end + 1 :]
            )
        else:
            print("Error: Could not find place to insert CI badges", file=sys.stderr)
            return

    with open(readme_path, "w") as f:
        f.write(new_content)

    print(f"Updated {readme_path} with CI status badges")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(
        description="Generate CI status badges from GitHub workflows"
    )
    parser.add_argument(
        "--repo", default="KooshaPari/Dino", help="GitHub repository (owner/repo)"
    )
    parser.add_argument(
        "--token", default=None, help="GitHub API token (default: GITHUB_TOKEN env var)"
    )
    parser.add_argument(
        "--workflows",
        default="ci,build-gate,lint",
        help="Comma-separated workflow names (default: ci,build-gate,lint)",
    )
    parser.add_argument("--output", default="README.md", help="Output file")
    parser.add_argument(
        "--json", action="store_true", help="Output as JSON instead of updating README"
    )

    args = parser.parse_args()

    client = GitHubWorkflowClient(args.repo, args.token)
    workflows = [w.strip() for w in args.workflows.split(",")]

    if args.json:
        results = {}
        for workflow in workflows:
            status = client.get_workflow_status(workflow)
            results[workflow] = status or {"error": "Not found"}
        print(json.dumps(results, indent=2))
    else:
        badge_section = generate_badge_section(client, workflows)
        update_readme(badge_section, args.repo)
