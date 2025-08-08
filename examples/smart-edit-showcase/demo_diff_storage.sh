#!/bin/bash
# Demo: Smart Edit Diff Storage System
# Shows how .st folder stores file diffs automatically

set -e

YELLOW='\033[1;33m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
RED='\033[0;31m'
MAGENTA='\033[0;35m'
NC='\033[0m'

echo -e "${YELLOW}🗂️ Smart Edit Diff Storage Demo${NC}"
echo -e "${YELLOW}================================${NC}\n"

# Check if .st exists
if [ ! -d ".st" ]; then
    echo -e "${CYAN}Creating .st folder for diff storage...${NC}"
    mkdir -p .st
    echo ".st/" >> .gitignore
    echo -e "${GREEN}✓ Created .st folder and updated .gitignore${NC}\n"
fi

# Simulate file edits
echo -e "${BLUE}📝 Let's simulate some edits to user_service.rs${NC}\n"

# Original content
echo -e "${CYAN}Original file state:${NC}"
head -10 src/user_service.rs
echo "..."

# Backup original
cp src/user_service.rs .st/user_service.rs

# Edit 1: Add a function
echo -e "\n${YELLOW}Edit 1: Adding delete_user function${NC}"
TIMESTAMP1=$(date +%s)
cat >> src/user_service.rs << 'EOF'

    pub fn delete_user(&mut self, id: u64) -> Option<User> {
        self.users.remove(&id)
    }
EOF

# Create diff
diff -u .st/user_service.rs src/user_service.rs > .st/src-user_service.rs-$TIMESTAMP1 || true
echo -e "${GREEN}✓ Diff saved to: .st/src-user_service.rs-$TIMESTAMP1${NC}"

sleep 1

# Edit 2: Add another function
echo -e "\n${YELLOW}Edit 2: Adding list_users function${NC}"
TIMESTAMP2=$(date +%s)
cat >> src/user_service.rs << 'EOF'

    pub fn list_users(&self) -> Vec<&User> {
        self.users.values().collect()
    }
EOF

# Create diff
diff -u .st/user_service.rs src/user_service.rs > .st/src-user_service.rs-$TIMESTAMP2 || true
echo -e "${GREEN}✓ Diff saved to: .st/src-user_service.rs-$TIMESTAMP2${NC}"

# Show diff storage
echo -e "\n${CYAN}📊 Diff Storage Status:${NC}"
echo -e "${BLUE}Contents of .st folder:${NC}"
ls -la .st/

echo -e "\n${MAGENTA}📈 Storage Statistics:${NC}"
DIFF_COUNT=$(ls .st/src-user_service.rs-* 2>/dev/null | wc -l)
TOTAL_SIZE=$(du -sh .st 2>/dev/null | cut -f1)
echo "Total diffs stored: $DIFF_COUNT"
echo "Storage size: $TOTAL_SIZE"

# Show a diff
if [ $DIFF_COUNT -gt 0 ]; then
    echo -e "\n${YELLOW}📄 Sample diff content:${NC}"
    LATEST_DIFF=$(ls -t .st/src-user_service.rs-* | head -1)
    echo "From: $LATEST_DIFF"
    echo "---"
    head -20 "$LATEST_DIFF"
    echo "..."
fi

# Demonstrate reconstruction
echo -e "\n${GREEN}🔄 File History:${NC}"
echo "With .st diffs, we can:"
echo "• Track all changes made by Smart Edit"
echo "• See exactly what was modified and when"
echo "• Reconstruct file state at any point"
echo "• Audit AI-assisted changes"
echo "• Roll back if needed"

# Restore original
echo -e "\n${CYAN}Restoring original file for demo cleanliness...${NC}"
cp .st/user_service.rs src/user_service.rs
echo -e "${GREEN}✓ Original file restored${NC}"

echo -e "\n${YELLOW}✨ Benefits of Diff Storage:${NC}"
echo "1. ${GREEN}Local audit trail${NC} - Track all AI edits"
echo "2. ${GREEN}Git-independent${NC} - Works even without commits"  
echo "3. ${GREEN}Timestamp tracking${NC} - Know exactly when changes happened"
echo "4. ${GREEN}Easy rollback${NC} - Restore any previous version"
echo "5. ${GREEN}Minimal overhead${NC} - Only stores diffs, not full files"

echo -e "\n${BLUE}This is why Smart Tree's diff storage is essential for AI-assisted development!${NC}"
echo -e "${MAGENTA}Every edit is tracked, every change is auditable.${NC}\n"