#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = [
#   "questionary==2.1.1",
#   "semver==3.0.4",
# ]
# ///

from __future__ import annotations

import argparse
import shlex
import shutil
import subprocess
import sys
import tomllib
from pathlib import Path
from typing import Optional, Sequence

import questionary
from questionary import Choice, Style
from semver import Version


ROOT = Path(__file__).resolve().parent.parent
VERSION_PATH = ROOT / "VERSION"
CARGO_PATH = ROOT / "Cargo.toml"
CARGO_LOCK_PATH = ROOT / "Cargo.lock"
REMOTE = "origin"

PROMPT_STYLE = Style(
  [
    ("qmark", "fg:#00a67d bold"),
    ("question", "bold"),
    ("answer", "fg:#00a67d bold"),
    ("pointer", "fg:#00a67d bold"),
    ("highlighted", "fg:#00a67d bold"),
    ("selected", "fg:#00a67d"),
  ]
)


class ReleaseError(RuntimeError):
  pass


def command(
  *args: str,
  capture: bool = False,
  check: bool = True,
) -> subprocess.CompletedProcess[str]:
  return subprocess.run(
    args,
    cwd=ROOT,
    check=check,
    capture_output=capture,
    text=True,
  )


def output(*args: str) -> str:
  return command(*args, capture=True).stdout.strip()


def require_command(name: str) -> None:
  if shutil.which(name) is None:
    raise ReleaseError(f"required command not found: {name}")


def ensure_repository_is_ready() -> None:
  for name in ("cargo", "git", "python3"):
    require_command(name)

  try:
    repository = Path(output("git", "rev-parse", "--show-toplevel")).resolve()
  except subprocess.CalledProcessError as error:
    raise ReleaseError("script must run from a Git repository") from error

  if repository != ROOT:
    raise ReleaseError("script must run from the feth-fixed-growths repository")
  if output("git", "branch", "--show-current") != "main":
    raise ReleaseError("switch to the main branch before releasing")
  if command("git", "remote", "get-url", REMOTE, capture=True, check=False).returncode:
    raise ReleaseError(f"the {REMOTE} remote is not configured")
  if output("git", "status", "--porcelain"):
    raise ReleaseError("working tree is not clean; commit or stash changes first")

  print(f"Fetching {REMOTE}/main and release tags...", flush=True)
  command("git", "fetch", REMOTE, f"main:refs/remotes/{REMOTE}/main", "--tags")
  if output("git", "rev-parse", "HEAD") != output(
    "git", "rev-parse", f"refs/remotes/{REMOTE}/main"
  ):
    raise ReleaseError(f"local main is not synchronized with {REMOTE}/main")


def read_version() -> Version:
  raw_version = VERSION_PATH.read_text(encoding="utf-8").strip()
  try:
    version = Version.parse(raw_version)
  except ValueError as error:
    raise ReleaseError(f"VERSION is not a SemVer value: {raw_version}") from error
  if version.prerelease is not None or version.build is not None:
    raise ReleaseError(f"VERSION must be a stable SemVer value: {raw_version}")

  with CARGO_PATH.open("rb") as cargo_file:
    cargo_version = tomllib.load(cargo_file)["package"]["version"]
  if cargo_version != str(version):
    raise ReleaseError(
      f"Cargo package version {cargo_version} does not match VERSION {version}"
    )
  return version


def next_versions(version: Version) -> dict[str, Version]:
  return {
    "patch": version.bump_patch(),
    "minor": version.bump_minor(),
    "major": version.bump_major(),
  }


def plain_select(version: Version, candidates: dict[str, Version]) -> Optional[str]:
  print(f"\nCurrent version: v{version}\n")
  print(f"  1) patch  v{candidates['patch']}")
  print(f"  2) minor  v{candidates['minor']}")
  print(f"  3) major  v{candidates['major']}")
  print("  q) cancel\n")
  try:
    selection = input("Select release type [1-3/q]: ").strip().lower()
  except EOFError:
    return None
  return {
    "1": "patch",
    "patch": "patch",
    "2": "minor",
    "minor": "minor",
    "3": "major",
    "major": "major",
    "q": None,
    "": None,
  }.get(selection, "invalid")


def select_bump(version: Version, candidates: dict[str, Version]) -> Optional[str]:
  if not sys.stdin.isatty():
    selection = plain_select(version, candidates)
    if selection == "invalid":
      raise ReleaseError("unknown release type")
    return selection

  selection = questionary.select(
    f"Select the next version (current: v{version})",
    choices=[
      Choice(f"patch  v{candidates['patch']}", value="patch"),
      Choice(f"minor  v{candidates['minor']}", value="minor"),
      Choice(f"major  v{candidates['major']}", value="major"),
      Choice("Cancel", value="cancel"),
    ],
    style=PROMPT_STYLE,
    instruction="(use arrow keys)",
  ).ask()
  return None if selection in {None, "cancel"} else selection


def confirm_release(tag: str) -> bool:
  if not sys.stdin.isatty():
    try:
      answer = input("Continue? [y/N]: ").strip().lower()
    except EOFError:
      return False
    return answer in {"y", "yes"}

  return (
    questionary.confirm(
      f"Create and push release {tag}?",
      default=False,
      style=PROMPT_STYLE,
    ).ask()
    is True
  )


def tag_exists(tag: str) -> bool:
  return (
    command(
      "git",
      "rev-parse",
      "--verify",
      "--quiet",
      f"refs/tags/{tag}",
      capture=True,
      check=False,
    ).returncode
    == 0
  )


def write_cargo_version(version: Version) -> None:
  lines = CARGO_PATH.read_text(encoding="utf-8").splitlines(keepends=True)
  in_package = False
  replaced = False
  for index, line in enumerate(lines):
    stripped = line.strip()
    if stripped == "[package]":
      in_package = True
      continue
    if in_package and stripped.startswith("["):
      break
    if in_package and stripped.startswith("version = "):
      lines[index] = f'version = "{version}"\n'
      replaced = True
      break
  if not replaced:
    raise ReleaseError("Cargo.toml has no package version")
  CARGO_PATH.write_text("".join(lines), encoding="utf-8")


def rollback_version_files() -> None:
  command(
    "git",
    "restore",
    "--staged",
    "--worktree",
    "--",
    "VERSION",
    "Cargo.toml",
    "Cargo.lock",
    capture=True,
    check=False,
  )
  print("Restored version files.", file=sys.stderr)


def run_release_checks() -> None:
  print("\nChecking formatting...", flush=True)
  command("cargo", "fmt", "--check")
  print("Running host tests...", flush=True)
  command("cargo", "test", "--locked")
  print("Checking the Skyline target...", flush=True)
  command("cargo", "skyline", "check")
  print("Building the release NRO...", flush=True)
  command("cargo", "skyline", "build", "--release")
  command(
    "python3",
    "tools/verify_nro.py",
    "target/aarch64-skyline-switch/release/libfeth_fixed_growths.nro",
  )


def publish(bump: Optional[str]) -> int:
  ensure_repository_is_ready()
  current_version = read_version()
  candidates = next_versions(current_version)
  selected_bump = bump or select_bump(current_version, candidates)
  if selected_bump is None:
    print("Release cancelled.")
    return 0

  next_version = candidates[selected_bump]
  tag = f"v{next_version}"
  if tag_exists(tag):
    raise ReleaseError(f"tag already exists: {tag}")

  print("\nRelease plan:")
  print(f"  version: v{current_version} -> {tag}")
  print(f"  commit:  chore: release {tag}")
  print(f"  remote:  {REMOTE}/main and {tag}\n")
  if not confirm_release(tag):
    print("Release cancelled.")
    return 0

  run_release_checks()

  version_changed = False
  commit_created = False
  try:
    VERSION_PATH.write_text(f"{next_version}\n", encoding="utf-8")
    write_cargo_version(next_version)
    version_changed = True
    command("cargo", "check")
    command("git", "add", "VERSION", "Cargo.toml", "Cargo.lock")

    changed_files = set(output("git", "diff", "--cached", "--name-only").splitlines())
    expected_files = {"VERSION", "Cargo.toml", "Cargo.lock"}
    if changed_files != expected_files:
      raise ReleaseError(
        "release commit must contain exactly VERSION, Cargo.toml, and Cargo.lock"
      )

    command("git", "commit", "-m", f"chore: release {tag}")
    commit_created = True
    command("git", "push", REMOTE, "main")
    command("git", "tag", "-a", tag, "-m", tag)
    command("git", "push", REMOTE, tag)
  except BaseException:
    if version_changed and not commit_created:
      rollback_version_files()
    raise

  print(f"\nPublished {tag}.")
  print(
    "Release workflow: "
    "https://github.com/jinghaihan/feth-fixed-growths/actions/workflows/release.yml"
  )
  return 0


def parse_args(argv: Optional[Sequence[str]] = None) -> argparse.Namespace:
  parser = argparse.ArgumentParser(
    description="Create and push a FETH Fixed Growths SemVer release.",
  )
  parser.add_argument(
    "bump",
    nargs="?",
    choices=("major", "minor", "patch"),
    help="release type; omit to select interactively",
  )
  return parser.parse_args(argv)


def main() -> int:
  args = parse_args()
  try:
    return publish(args.bump)
  except KeyboardInterrupt:
    print("\nRelease cancelled.", file=sys.stderr)
    return 130
  except ReleaseError as error:
    print(f"release failed: {error}", file=sys.stderr)
    return 1
  except subprocess.CalledProcessError as error:
    print(
      f"release failed: command exited with {error.returncode}: "
      f"{shlex.join(error.cmd)}",
      file=sys.stderr,
    )
    if error.stderr:
      print(error.stderr.strip(), file=sys.stderr)
    return error.returncode or 1


if __name__ == "__main__":
  raise SystemExit(main())
