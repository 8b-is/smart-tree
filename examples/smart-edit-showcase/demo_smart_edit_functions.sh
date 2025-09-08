#!/bin/bash
# Smart Edit Function Tools Demonstration
# Shows the power of AST-aware editing with 90-95% token reduction!

set -e

YELLOW='\033[1;33m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${YELLOW}🎯 Smart Edit Function Tools Showcase${NC}"
echo -e "${YELLOW}=====================================\n${NC}"

echo -e "${CYAN}📁 Project Structure:${NC}"
echo "smart-edit-showcase/"
echo "├── src/"
echo "│   ├── user_service.rs  (41 lines, ~800 tokens)"
echo "│   ├── auth_handler.rs  (17 lines, ~300 tokens)"
echo "│   └── lib.rs          (6 lines, ~100 tokens)"
echo ""

echo -e "${GREEN}Let's demonstrate various smart edit operations...${NC}\n"

# Function to simulate Smart Edit API calls
simulate_smart_edit() {
    local operation=$1
    local file=$2
    local tokens=$3
    local traditional_tokens=$4
    
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${YELLOW}Operation: $operation${NC}"
    echo -e "File: $file"
    echo -e "${GREEN}Smart Edit Tokens: $tokens${NC} vs ${RED}Traditional: $traditional_tokens${NC}"
    echo -e "Savings: $(( 100 - (tokens * 100 / traditional_tokens) ))% reduction! 🚀"
    echo ""
}

# Demo 1: Insert a new function
echo -e "${CYAN}1️⃣ INSERT FUNCTION${NC}"
echo "Task: Add a delete_user function to UserService"
echo ""
echo "Smart Edit Request:"
echo '```json'
echo '{
  "operation": "InsertFunction",
  "name": "delete_user",
  "after": "get_user",
  "body": "pub fn delete_user(&mut self, id: u64) -> Option<User> {
        self.users.remove(&id)
    }"
}'
echo '```'
simulate_smart_edit "InsertFunction" "user_service.rs" 35 850

# Demo 2: Replace function body
echo -e "${CYAN}2️⃣ REPLACE FUNCTION BODY${NC}"
echo "Task: Improve the verify_token implementation"
echo ""
echo "Smart Edit Request:"
echo '```json'
echo '{
  "operation": "ReplaceFunction",
  "name": "verify_token",
  "new_body": "use jwt::{decode, Validation};
        
        match decode::<Claims>(token, &self.secret_key, &Validation::default()) {
            Ok(_) => true,
            Err(_) => false,
        }"
}'
echo '```'
simulate_smart_edit "ReplaceFunction" "auth_handler.rs" 45 320

# Demo 3: Add imports
echo -e "${CYAN}3️⃣ ADD IMPORTS${NC}"
echo "Task: Add JWT library imports to auth_handler.rs"
echo ""
echo "Smart Edit Request:"
echo '```json'
echo '{
  "operation": "AddImport",
  "import": "use jwt::{encode, decode, Header, Validation, EncodingKey, DecodingKey};"
}'
echo '```'
simulate_smart_edit "AddImport" "auth_handler.rs" 22 320

# Demo 4: Add method to struct
echo -e "${CYAN}4️⃣ ADD METHOD TO STRUCT${NC}"
echo "Task: Add update_user method to UserService"
echo ""
echo "Smart Edit Request:"
echo '```json'
echo '{
  "operation": "AddMethod",
  "class_name": "UserService",
  "name": "update_user",
  "after": "create_user",
  "body": "pub fn update_user(&mut self, id: u64, name: String, email: String) -> Option<&User> {
        if let Some(user) = self.users.get_mut(&id) {
            user.name = name;
            user.email = email;
            Some(user)
        } else {
            None
        }
    }"
}'
echo '```'
simulate_smart_edit "AddMethod" "user_service.rs" 55 900

# Demo 5: Extract function
echo -e "${CYAN}5️⃣ EXTRACT FUNCTION${NC}"
echo "Task: Extract user validation logic into separate function"
echo ""
echo "Smart Edit Request:"
echo '```json'
echo '{
  "operation": "ExtractFunction",
  "from": "create_user",
  "lines": "26-28",
  "to": "validate_user_data",
  "body": "fn validate_user_data(name: &str, email: &str) -> Result<(), String> {
        if name.is_empty() {
            return Err(\"Name cannot be empty\".to_string());
        }
        if !email.contains(\"@\") {
            return Err(\"Invalid email format\".to_string());
        }
        Ok(())
    }"
}'
echo '```'
simulate_smart_edit "ExtractFunction" "user_service.rs" 68 950

# Demo 6: Batch operations
echo -e "${CYAN}6️⃣ BATCH OPERATIONS${NC}"
echo "Task: Add error handling, logging, and metrics in one go"
echo ""
echo "Smart Edit Request:"
echo '```json'
echo '{
  "edits": [
    {
      "operation": "AddImport",
      "import": "use log::{info, error};"
    },
    {
      "operation": "AddImport",
      "import": "use metrics::{counter, histogram};"
    },
    {
      "operation": "WrapCode",
      "function": "create_user",
      "wrapper": "histogram!(\"user_creation_time\").record(|| { ... })"
    }
  ]
}'
echo '```'
simulate_smart_edit "Batch Edit (3 operations)" "user_service.rs" 75 1000

echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}📊 TOTAL SUMMARY${NC}"
echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo "Traditional Method (sending full files):"
echo -e "  Total tokens used: ${RED}4,340 tokens${NC}"
echo ""
echo "Smart Edit Method (AST-aware operations):"
echo -e "  Total tokens used: ${GREEN}300 tokens${NC}"
echo ""
echo -e "${CYAN}✨ Total Savings: 93% token reduction!${NC}"
echo ""
echo -e "${YELLOW}🎯 Key Benefits:${NC}"
echo "  • Faster API calls (less data transfer)"
echo "  • Lower costs (fewer tokens)"
echo "  • Reduced latency (smaller payloads)"
echo "  • Better accuracy (surgical precision)"
echo "  • Context preservation (no file corruption)"
echo ""
echo -e "${GREEN}🚀 This is why Smart Tree's editing tools are revolutionary!${NC}"
echo ""
echo -e "${BLUE}Try the interactive demo: python3 demo_smart_edit_interactive.py${NC}"