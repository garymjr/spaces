#!/bin/bash
# Integration test for hooks system

set -e

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m'

# Setup
TEST_DIR=$(mktemp -d)
cd "$TEST_DIR"
git init --initial-branch=main
echo "test" > test.txt
git add test.txt
git commit -m "initial commit"

echo "Testing in $TEST_DIR"

# Create hooks directory
mkdir -p .spaces/hooks

# Create test hooks
cat > .spaces/hooks/pre-create << 'EOF'
#!/bin/bash
echo "PRE_CREATE: $@" > /tmp/test-hooks.log
echo "pre-create hook executed"
EOF
chmod +x .spaces/hooks/pre-create

cat > .spaces/hooks/post-create << 'EOF'
#!/bin/bash
echo "POST_CREATE: $@" >> /tmp/test-hooks.log
echo "post-create hook executed"
EOF
chmod +x .spaces/hooks/post-create

cat > .spaces/hooks/pre-enter << 'EOF'
#!/bin/bash
echo "PRE_ENTER: $@" >> /tmp/test-hooks.log
echo "pre-enter hook executed"
EOF
chmod +x .spaces/hooks/pre-enter

cat > .spaces/hooks/post-enter << 'EOF'
#!/bin/bash
echo "POST_ENTER: $@" >> /tmp/test-hooks.log
echo "post-enter hook executed"
EOF
chmod +x .spaces/hooks/post-enter

cat > .spaces/hooks/pre-remove << 'EOF'
#!/bin/bash
echo "PRE_REMOVE: $@" >> /tmp/test-hooks.log
echo "pre-remove hook executed"
EOF
chmod +x .spaces/hooks/pre-remove

cat > .spaces/hooks/post-remove << 'EOF'
#!/bin/bash
echo "POST_REMOVE: $@" >> /tmp/test-hooks.log
echo "post-remove hook executed"
EOF
chmod +x .spaces/hooks/post-remove

# Clear previous test logs
> /tmp/test-hooks.log

# Run spaces commands (use the built binary)
SPACES="/Users/gmurray/Developer/spaces/zig-out/bin/spaces"

if [ ! -f "$SPACES" ]; then
    echo "Binary not found. Building..."
    cd /Users/gmurray/Developer/spaces
    just build
    cd "$TEST_DIR"
fi

echo ""
echo "=== Testing create hooks ==="
$SPACES create test-worktree || {
    echo -e "${RED}Failed to create worktree${NC}"
    exit 1
}

echo ""
echo "=== Testing enter hooks ==="
$SPACES enter test-worktree > /dev/null || {
    echo -e "${RED}Failed to enter worktree${NC}"
    exit 1
}

echo ""
echo "=== Testing remove hooks ==="
$SPACES remove test-worktree || {
    echo -e "${RED}Failed to remove worktree${NC}"
    exit 1
}

echo ""
echo "=== Checking hook execution ==="
echo "Hook log:"
cat /tmp/test-hooks.log

# Check all hooks were executed
if grep -q "PRE_CREATE" /tmp/test-hooks.log && \
   grep -q "POST_CREATE" /tmp/test-hooks.log && \
   grep -q "PRE_ENTER" /tmp/test-hooks.log && \
   grep -q "POST_ENTER" /tmp/test-hooks.log && \
   grep -q "PRE_REMOVE" /tmp/test-hooks.log && \
   grep -q "POST_REMOVE" /tmp/test-hooks.log; then
    echo -e "${GREEN}✓ All hooks executed successfully!${NC}"
    exit 0
else
    echo -e "${RED}✗ Some hooks did not execute${NC}"
    exit 1
fi
