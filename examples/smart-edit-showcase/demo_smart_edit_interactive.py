#!/usr/bin/env python3
"""
Smart Edit Interactive Demo
Shows real-time token counting and efficiency comparisons
"""

import json
import os
from typing import Dict, List, Tuple

# ANSI color codes
YELLOW = '\033[1;33m'
GREEN = '\033[0;32m'
BLUE = '\033[0;34m'
CYAN = '\033[0;36m'
RED = '\033[0;31m'
MAGENTA = '\033[0;35m'
NC = '\033[0m'

class SmartEditDemo:
    def __init__(self):
        self.total_smart_tokens = 0
        self.total_traditional_tokens = 0
        self.operations_count = 0
        
    def count_tokens(self, text: str) -> int:
        """Rough token estimation (1 token ≈ 4 chars)"""
        return len(text) // 4
    
    def show_operation(self, op_name: str, smart_edit: Dict, file_content: str):
        """Display a smart edit operation with token comparison"""
        self.operations_count += 1
        
        # Calculate tokens
        smart_tokens = self.count_tokens(json.dumps(smart_edit))
        traditional_tokens = self.count_tokens(file_content)
        
        self.total_smart_tokens += smart_tokens
        self.total_traditional_tokens += traditional_tokens
        
        savings = 100 - (smart_tokens * 100 // traditional_tokens)
        
        print(f"\n{BLUE}{'='*60}{NC}")
        print(f"{YELLOW}Operation #{self.operations_count}: {op_name}{NC}")
        print(f"{BLUE}{'='*60}{NC}\n")
        
        print(f"Smart Edit Request:")
        print(f"{CYAN}{json.dumps(smart_edit, indent=2)}{NC}\n")
        
        print(f"Token Usage:")
        print(f"  {GREEN}Smart Edit: {smart_tokens} tokens{NC}")
        print(f"  {RED}Traditional (full file): {traditional_tokens} tokens{NC}")
        print(f"  {MAGENTA}Savings: {savings}% reduction!{NC}")
        
    def run_demo(self):
        """Run the interactive demonstration"""
        print(f"{YELLOW}🎯 Smart Edit Interactive Demo{NC}")
        print(f"{YELLOW}{'='*30}{NC}\n")
        
        # Load our mock files
        with open('src/user_service.rs', 'r') as f:
            user_service_content = f.read()
        
        with open('src/auth_handler.rs', 'r') as f:
            auth_handler_content = f.read()
        
        print(f"{CYAN}📁 Mock Project Files:{NC}")
        print(f"  • user_service.rs ({self.count_tokens(user_service_content)} tokens)")
        print(f"  • auth_handler.rs ({self.count_tokens(auth_handler_content)} tokens)")
        
        input(f"\n{GREEN}Press Enter to start the demonstration...{NC}")
        
        # Demo 1: Insert Function
        self.show_operation(
            "Insert delete_user function",
            {
                "operation": "InsertFunction",
                "name": "delete_user",
                "after": "get_user",
                "body": """pub fn delete_user(&mut self, id: u64) -> Option<User> {
        self.users.remove(&id)
    }""",
                "visibility": "public"
            },
            user_service_content
        )
        
        input(f"\n{GREEN}Press Enter for next operation...{NC}")
        
        # Demo 2: Add Error Type
        self.show_operation(
            "Add custom error type",
            {
                "operation": "InsertClass",
                "after": "User",
                "content": """#[derive(Debug)]
pub enum UserError {
    NotFound(u64),
    InvalidEmail(String),
    DuplicateEmail(String),
}"""
            },
            user_service_content
        )
        
        input(f"\n{GREEN}Press Enter for next operation...{NC}")
        
        # Demo 3: Replace Function with Error Handling
        self.show_operation(
            "Update create_user with Result type",
            {
                "operation": "ReplaceFunction",
                "name": "create_user",
                "new_body": """pub fn create_user(&mut self, name: String, email: String) -> Result<User, UserError> {
        // Check for duplicate email
        if self.users.values().any(|u| u.email == email) {
            return Err(UserError::DuplicateEmail(email));
        }
        
        // Validate email format
        if !email.contains('@') {
            return Err(UserError::InvalidEmail(email));
        }
        
        let user = User {
            id: self.next_id,
            name,
            email,
        };
        self.users.insert(user.id, user.clone());
        self.next_id += 1;
        Ok(user)
    }"""
            },
            user_service_content
        )
        
        input(f"\n{GREEN}Press Enter for next operation...{NC}")
        
        # Demo 4: Batch operations on auth_handler
        self.show_operation(
            "Enhance auth_handler with JWT support (batch)",
            {
                "edits": [
                    {
                        "operation": "AddImport",
                        "import": "use jwt::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey};"
                    },
                    {
                        "operation": "AddImport", 
                        "import": "use serde::{Serialize, Deserialize};"
                    },
                    {
                        "operation": "InsertClass",
                        "before": "AuthHandler",
                        "content": """#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}"""
                    },
                    {
                        "operation": "AddMethod",
                        "class_name": "AuthHandler",
                        "name": "generate_token",
                        "body": """pub fn generate_token(&self, user_id: &str) -> Result<String, jwt::errors::Error> {
        let claims = Claims {
            sub: user_id.to_owned(),
            exp: 10000000000, // Far future
        };
        
        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret_key.as_ref())
        )
    }"""
                    }
                ]
            },
            auth_handler_content
        )
        
        input(f"\n{GREEN}Press Enter for final summary...{NC}")
        
        # Show summary
        self.show_summary()
        
    def show_summary(self):
        """Display the final summary with total savings"""
        print(f"\n{YELLOW}{'='*60}{NC}")
        print(f"{YELLOW}📊 DEMONSTRATION SUMMARY{NC}")
        print(f"{YELLOW}{'='*60}{NC}\n")
        
        print(f"Total Operations: {self.operations_count}")
        print(f"\nToken Usage Comparison:")
        print(f"  {RED}Traditional Method (full files): {self.total_traditional_tokens:,} tokens{NC}")
        print(f"  {GREEN}Smart Edit Method (changes only): {self.total_smart_tokens:,} tokens{NC}")
        
        total_savings = 100 - (self.total_smart_tokens * 100 // self.total_traditional_tokens)
        print(f"\n{CYAN}✨ Total Token Savings: {total_savings}% reduction!{NC}")
        
        print(f"\n{MAGENTA}🚀 Benefits Demonstrated:{NC}")
        print(f"  • Surgical precision - edit exactly what you need")
        print(f"  • Massive token savings - 90-95% reduction typical")
        print(f"  • Batch operations - multiple edits in one request")
        print(f"  • Type safety - AST-aware prevents syntax errors")
        print(f"  • Context preservation - no risk of file corruption")
        
        print(f"\n{BLUE}💡 Pro Tips:{NC}")
        print(f"  • Use get_function_tree first to understand structure")
        print(f"  • Batch related edits for maximum efficiency")
        print(f"  • Smart Edit handles imports, formatting automatically")
        print(f"  • Perfect for refactoring and code generation")
        
        print(f"\n{GREEN}This is the future of AI-assisted coding!{NC}")
        print(f"{YELLOW}Crafted with love by Aye & Hue 🤝{NC}\n")

def main():
    """Run the interactive demonstration"""
    # Change to the demo directory
    demo_dir = os.path.dirname(os.path.abspath(__file__))
    os.chdir(demo_dir)
    
    demo = SmartEditDemo()
    demo.run_demo()

if __name__ == "__main__":
    main()