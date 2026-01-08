#!/bin/bash
# Example post-create hook
# This script runs after a new worktree is created
# Environment variables will be available with context about the worktree

echo "Post-create hook executed!"
echo "Setting up new worktree..."

# Example: Create a .env file
# cat > "$WORKTREE_PATH/.env" << 'EOF'
# ENV=development
# EOF

# Example: Install dependencies if needed
# if [ -f "package.json" ]; then
#     npm install
# fi

# Example: Run initialization scripts
# if [ -f "scripts/init.sh" ]; then
#     bash scripts/init.sh
# fi

echo "Setup complete!"
