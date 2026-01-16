#!/usr/bin/env bats
#
# Integration tests for vamp swarm merge workflow
#
# These tests use real git repositories to verify merge behavior

load '../helpers/test_helper'

# Setup a test repo with swarm worktrees before each test
setup() {
    setup_temp_dir

    # Create main repository
    TEST_REPO_DIR="$TEST_TEMP_DIR/test-repo"
    mkdir -p "$TEST_REPO_DIR"
    cd "$TEST_REPO_DIR"

    git init --initial-branch=main
    echo ".vamp-workers/" > .gitignore
    echo "initial content" > README.md
    git add .
    git commit -m "Initial commit"

    # Create worktrees directory
    mkdir -p .vamp-workers

    export TEST_REPO_DIR
}

teardown() {
    cd /
    teardown_temp_dir
}

# Helper to create a swarm worktree with optional commit
create_worktree() {
    local name="$1"
    local branch="swarm/$name"
    local wt_dir="$TEST_REPO_DIR/.vamp-workers/$name"

    git -C "$TEST_REPO_DIR" worktree add "$wt_dir" -b "$branch"
}

# Helper to add a commit to a worktree
add_commit_to_worktree() {
    local name="$1"
    local filename="${2:-feature.txt}"
    local content="${3:-feature content}"
    local wt_dir="$TEST_REPO_DIR/.vamp-workers/$name"

    echo "$content" > "$wt_dir/$filename"
    git -C "$wt_dir" add .
    git -C "$wt_dir" commit -m "Add $filename"
}

# Helper to add dirty file to worktree
add_dirty_file() {
    local name="$1"
    local filename="${2:-dirty.txt}"
    local wt_dir="$TEST_REPO_DIR/.vamp-workers/$name"

    echo "uncommitted" > "$wt_dir/$filename"
}

# ============================================
# Happy Path Tests
# ============================================

@test "merge: single clean worker with commits" {
    create_worktree "worker-1"
    add_commit_to_worktree "worker-1" "feature1.txt"

    cd "$TEST_REPO_DIR"
    run_vamp swarm --merge --yes

    assert_success
    assert_output --partial "Merged swarm/worker-1"
    assert_output --partial "1 succeeded"

    # Verify file was merged
    assert_file_exists "$TEST_REPO_DIR/feature1.txt"
}

@test "merge: multiple clean workers with commits" {
    create_worktree "worker-1"
    create_worktree "worker-2"
    add_commit_to_worktree "worker-1" "feature1.txt"
    add_commit_to_worktree "worker-2" "feature2.txt"

    cd "$TEST_REPO_DIR"
    run_vamp swarm --merge --yes

    assert_success
    assert_output --partial "2 succeeded"

    # Verify both files were merged
    assert_file_exists "$TEST_REPO_DIR/feature1.txt"
    assert_file_exists "$TEST_REPO_DIR/feature2.txt"
}

@test "merge: no commits to merge" {
    create_worktree "worker-1"
    # Don't add any commits

    cd "$TEST_REPO_DIR"
    run_vamp swarm --merge --yes

    assert_success
    assert_output --partial "No branches have commits to merge"
}

# ============================================
# Uncommitted Changes Tests
# ============================================

@test "merge: skips dirty worker without --force" {
    create_worktree "worker-1"
    create_worktree "worker-2"
    add_commit_to_worktree "worker-1" "feature1.txt"
    add_commit_to_worktree "worker-2" "feature2.txt"
    add_dirty_file "worker-1"

    cd "$TEST_REPO_DIR"
    run_vamp swarm --merge --yes

    assert_success
    # Should skip worker-1, merge worker-2
    assert_output --partial "Skipped 1 branch"
    assert_output --partial "swarm/worker-1"
    assert_output --partial "Merged swarm/worker-2"

    # Only worker-2's file should be merged
    [ ! -f "$TEST_REPO_DIR/feature1.txt" ]
    assert_file_exists "$TEST_REPO_DIR/feature2.txt"
}

@test "merge: all workers dirty shows helpful message" {
    create_worktree "worker-1"
    create_worktree "worker-2"
    add_commit_to_worktree "worker-1" "feature1.txt"
    add_commit_to_worktree "worker-2" "feature2.txt"
    add_dirty_file "worker-1"
    add_dirty_file "worker-2"

    cd "$TEST_REPO_DIR"
    run_vamp swarm --merge --yes

    assert_success
    assert_output --partial "have commits to merge, but all have uncommitted changes"
    assert_output --partial "--force"
    assert_output --partial "--autostash"

    # Nothing should be merged
    [ ! -f "$TEST_REPO_DIR/feature1.txt" ]
    [ ! -f "$TEST_REPO_DIR/feature2.txt" ]
}

# ============================================
# --force Flag Tests
# ============================================

@test "merge --force: merges dirty workers" {
    create_worktree "worker-1"
    add_commit_to_worktree "worker-1" "feature1.txt"
    add_dirty_file "worker-1"

    cd "$TEST_REPO_DIR"
    run_vamp swarm --merge --force --yes

    assert_success
    assert_output --partial "will merge anyway with --force"
    assert_output --partial "Merged swarm/worker-1"
    assert_file_exists "$TEST_REPO_DIR/feature1.txt"
}

@test "merge --force: merges multiple dirty workers" {
    create_worktree "worker-1"
    create_worktree "worker-2"
    add_commit_to_worktree "worker-1" "feature1.txt"
    add_commit_to_worktree "worker-2" "feature2.txt"
    add_dirty_file "worker-1"
    add_dirty_file "worker-2"

    cd "$TEST_REPO_DIR"
    run_vamp swarm --merge --force --yes

    assert_success
    assert_output --partial "2 succeeded"
    assert_file_exists "$TEST_REPO_DIR/feature1.txt"
    assert_file_exists "$TEST_REPO_DIR/feature2.txt"
}

# ============================================
# --autostash Flag Tests
# ============================================

@test "merge --autostash: stashes and restores changes" {
    create_worktree "worker-1"
    add_commit_to_worktree "worker-1" "feature1.txt"
    add_dirty_file "worker-1" "dirty.txt"

    cd "$TEST_REPO_DIR"
    run_vamp swarm --merge --autostash --yes

    assert_success
    assert_output --partial "Stashing changes"
    assert_output --partial "Merged swarm/worker-1"
    assert_output --partial "Restoring stashed changes"
    assert_output --partial "Restored worker-1"

    # Feature should be merged
    assert_file_exists "$TEST_REPO_DIR/feature1.txt"

    # Dirty file should still exist in worktree
    assert_file_exists "$TEST_REPO_DIR/.vamp-workers/worker-1/dirty.txt"
}

@test "merge --autostash: handles multiple dirty workers" {
    create_worktree "worker-1"
    create_worktree "worker-2"
    add_commit_to_worktree "worker-1" "feature1.txt"
    add_commit_to_worktree "worker-2" "feature2.txt"
    add_dirty_file "worker-1" "dirty1.txt"
    add_dirty_file "worker-2" "dirty2.txt"

    cd "$TEST_REPO_DIR"
    run_vamp swarm --merge --autostash --yes

    assert_success
    assert_output --partial "Stashing changes in worker-1"
    assert_output --partial "Stashing changes in worker-2"
    assert_output --partial "2 succeeded"
    assert_output --partial "Restored worker-1"
    assert_output --partial "Restored worker-2"

    # Both features merged
    assert_file_exists "$TEST_REPO_DIR/feature1.txt"
    assert_file_exists "$TEST_REPO_DIR/feature2.txt"

    # Verify stash pop happened (dirty files should exist somewhere in worktrees)
    local dirty_count
    dirty_count=$(find "$TEST_REPO_DIR/.vamp-workers" -name "dirty*.txt" 2>/dev/null | wc -l | tr -d ' ')
    [ "$dirty_count" -eq 2 ]
}

# ============================================
# Ignorable Files Tests
# ============================================

@test "merge: ignores .db files" {
    create_worktree "worker-1"
    add_commit_to_worktree "worker-1" "feature1.txt"
    # Add ignorable .db file
    echo "cache" > "$TEST_REPO_DIR/.vamp-workers/worker-1/cache.db"

    cd "$TEST_REPO_DIR"
    run_vamp swarm --merge --yes

    assert_success
    # Should not mention uncommitted changes
    refute_output --partial "uncommitted changes"
    assert_output --partial "Merged swarm/worker-1"
}

@test "merge: ignores .coverage files" {
    create_worktree "worker-1"
    add_commit_to_worktree "worker-1" "feature1.txt"
    echo "coverage data" > "$TEST_REPO_DIR/.vamp-workers/worker-1/.coverage"

    cd "$TEST_REPO_DIR"
    run_vamp swarm --merge --yes

    assert_success
    refute_output --partial "uncommitted changes"
    assert_output --partial "Merged swarm/worker-1"
}

@test "merge: ignores __pycache__" {
    create_worktree "worker-1"
    add_commit_to_worktree "worker-1" "feature1.txt"
    mkdir -p "$TEST_REPO_DIR/.vamp-workers/worker-1/__pycache__"
    echo "bytecode" > "$TEST_REPO_DIR/.vamp-workers/worker-1/__pycache__/test.pyc"

    cd "$TEST_REPO_DIR"
    run_vamp swarm --merge --yes

    assert_success
    refute_output --partial "uncommitted changes"
    assert_output --partial "Merged swarm/worker-1"
}

@test "merge: does not ignore source files" {
    create_worktree "worker-1"
    add_commit_to_worktree "worker-1" "feature1.txt"
    # Add a non-ignorable source file
    echo "code" > "$TEST_REPO_DIR/.vamp-workers/worker-1/main.py"

    cd "$TEST_REPO_DIR"
    run_vamp swarm --merge --yes

    assert_success
    # Should mention uncommitted changes for .py file
    assert_output --partial "Skipped 1 branch"
}

# ============================================
# Edge Cases
# ============================================

@test "merge: no worktrees" {
    cd "$TEST_REPO_DIR"
    # Don't create any worktrees

    run_vamp swarm --merge --yes

    assert_success
    assert_output --partial "No swarm worktrees found"
}

@test "merge: empty worktrees directory" {
    mkdir -p "$TEST_REPO_DIR/.vamp-workers"
    cd "$TEST_REPO_DIR"

    run_vamp swarm --merge --yes

    assert_success
    assert_output --partial "No swarm worktrees found"
}
