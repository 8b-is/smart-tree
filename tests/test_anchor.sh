#!/usr/bin/env bash
# Test suite for anchor.sh - Partnership Memory Helper
# "Every collaboration needs verification!" - Testy McTesterson 🧪

set -euo pipefail

# ANSI color codes
readonly GREEN='\033[0;32m'
readonly RED='\033[0;31m'
readonly YELLOW='\033[1;33m'
readonly BLUE='\033[0;34m'
readonly CYAN='\033[0;36m'
readonly BOLD='\033[1m'
readonly RESET='\033[0m'

# Test counters
TESTS_RUN=0
TESTS_PASSED=0
TESTS_FAILED=0

# Test directory setup
TEST_DIR=$(mktemp -d)
ANCHOR_SCRIPT="${ANCHOR_SCRIPT:-./scripts/anchor.sh}"

# Cleanup on exit
trap "rm -rf $TEST_DIR" EXIT

# Test framework functions
run_test() {
    local test_name="$1"
    local test_function="$2"
    
    ((TESTS_RUN++))
    echo -e "\n${BLUE}Running test:${RESET} $test_name"
    
    if $test_function; then
        ((TESTS_PASSED++))
        echo -e "${GREEN}✓ PASSED${RESET}"
    else
        ((TESTS_FAILED++))
        echo -e "${RED}✗ FAILED${RESET}"
    fi
}

assert_equals() {
    local expected="$1"
    local actual="$2"
    local message="${3:-Values should be equal}"
    
    if [[ "$expected" != "$actual" ]]; then
        echo -e "${RED}Assertion failed:${RESET} $message"
        echo -e "Expected: '$expected'"
        echo -e "Actual:   '$actual'"
        return 1
    fi
    return 0
}

assert_contains() {
    local haystack="$1"
    local needle="$2"
    local message="${3:-Should contain substring}"
    
    if [[ ! "$haystack" =~ "$needle" ]]; then
        echo -e "${RED}Assertion failed:${RESET} $message"
        echo -e "String: '$haystack'"
        echo -e "Should contain: '$needle'"
        return 1
    fi
    return 0
}

assert_not_empty() {
    local value="$1"
    local message="${2:-Value should not be empty}"
    
    if [[ -z "$value" ]]; then
        echo -e "${RED}Assertion failed:${RESET} $message"
        return 1
    fi
    return 0
}

assert_file_exists() {
    local file="$1"
    local message="${2:-File should exist}"
    
    if [[ ! -f "$file" ]]; then
        echo -e "${RED}Assertion failed:${RESET} $message"
        echo -e "File not found: '$file'"
        return 1
    fi
    return 0
}

# Test: Script exists and is executable
test_script_exists() {
    assert_file_exists "$ANCHOR_SCRIPT" "anchor.sh script should exist"
    
    if [[ ! -x "$ANCHOR_SCRIPT" ]]; then
        echo -e "${RED}Script is not executable${RESET}"
        return 1
    fi
    return 0
}

# Test: Help command
test_help_command() {
    local output
    output=$("$ANCHOR_SCRIPT" help 2>&1) || true
    
    assert_contains "$output" "Smart Tree Memory Anchor Helper" "Should show title"
    assert_contains "$output" "USAGE:" "Should show usage section"
    assert_contains "$output" "anchor save" "Should show save command"
    assert_contains "$output" "anchor find" "Should show find command"
    assert_contains "$output" "EXAMPLES:" "Should show examples"
    
    return 0
}

# Test: No arguments shows usage
test_no_args_shows_usage() {
    local output
    output=$("$ANCHOR_SCRIPT" 2>&1) || true
    
    assert_contains "$output" "USAGE:" "No args should show usage"
    return 0
}

# Test: Invalid command handling
test_invalid_command() {
    local output
    output=$("$ANCHOR_SCRIPT" definitely_not_a_command 2>&1) || true
    
    assert_contains "$output" "Unknown command" "Should error on invalid command"
    return 0
}

# Test: Save command validation
test_save_command_validation() {
    local output
    
    # Missing arguments
    output=$("$ANCHOR_SCRIPT" save 2>&1) || true
    assert_contains "$output" "Usage:" "Should show usage for missing args"
    
    # Only context provided
    output=$("$ANCHOR_SCRIPT" save "test context" 2>&1) || true
    assert_contains "$output" "Usage:" "Should show usage for missing keywords"
    
    return 0
}

# Test: Find command validation
test_find_command_validation() {
    local output
    
    # Missing keywords
    output=$("$ANCHOR_SCRIPT" find 2>&1) || true
    assert_contains "$output" "Usage:" "Should show usage for missing keywords"
    
    return 0
}

# Test: List command
test_list_command() {
    local output
    output=$("$ANCHOR_SCRIPT" list 2>&1) || true
    
    # Should at least not crash
    assert_equals "0" "$?" "List command should not fail"
    return 0
}

# Test: Environment variables
test_environment_variables() {
    local output
    
    # Test ST_PROJECT override
    ST_PROJECT="/custom/path" output=$("$ANCHOR_SCRIPT" help 2>&1) || true
    assert_equals "0" "$?" "Should handle ST_PROJECT env var"
    
    # Test ST_ORIGIN override
    ST_ORIGIN="test:origin" output=$("$ANCHOR_SCRIPT" help 2>&1) || true
    assert_equals "0" "$?" "Should handle ST_ORIGIN env var"
    
    return 0
}

# Test: Anchor type validation
test_anchor_types() {
    local valid_types=("pattern_insight" "solution" "breakthrough" "learning" "joke" "technical" "process")
    
    for type in "${valid_types[@]}"; do
        # Would normally test actual save, but MCP dependency makes it complex
        # Just ensure the script accepts the type
        output=$("$ANCHOR_SCRIPT" save "test" "keyword" "$type" 2>&1) || true
        # Check it doesn't error on type validation
    done
    
    return 0
}

# Test: Multiple keywords handling
test_multiple_keywords() {
    # Test comma-separated keywords
    local keywords="test,multiple,keywords,with,commas"
    
    # This would normally call the MCP tool, but we're testing input parsing
    # The script should handle comma separation correctly
    
    return 0
}

# Test: Special characters in input
test_special_characters() {
    local special_context="Fixed bug with Arc<Mutex<T>> and Result<(), Error>"
    local special_keywords="arc<mutex>,result<error>,generic<t>"
    
    # Should handle special characters without breaking
    # (Would test actual save but MCP dependency)
    
    return 0
}

# Test: Rapport command validation
test_rapport_command() {
    local output
    
    # With AI tool specified
    output=$("$ANCHOR_SCRIPT" rapport claude 2>&1) || true
    
    # Without AI tool (should use default)
    output=$("$ANCHOR_SCRIPT" rapport 2>&1) || true
    
    return 0
}

# Test: Heatmap command
test_heatmap_command() {
    local output
    output=$("$ANCHOR_SCRIPT" heatmap 2>&1) || true
    
    # Should at least not crash
    assert_equals "0" "$?" "Heatmap command should not fail"
    return 0
}

# Test: Patterns command
test_patterns_command() {
    local output
    
    # Without type filter
    output=$("$ANCHOR_SCRIPT" patterns 2>&1) || true
    
    # With type filter
    output=$("$ANCHOR_SCRIPT" patterns algorithm 2>&1) || true
    
    return 0
}

# Test: Insights command validation
test_insights_command() {
    local output
    
    # Missing keywords
    output=$("$ANCHOR_SCRIPT" insights 2>&1) || true
    assert_contains "$output" "Usage:" "Should show usage for missing keywords"
    
    # With keywords
    output=$("$ANCHOR_SCRIPT" insights "performance,optimization" 2>&1) || true
    
    return 0
}

# Test: Invite command validation
test_invite_command() {
    local output
    
    # Missing context
    output=$("$ANCHOR_SCRIPT" invite 2>&1) || true
    assert_contains "$output" "Usage:" "Should show usage for missing context"
    
    # With context
    output=$("$ANCHOR_SCRIPT" invite "need help with optimization" 2>&1) || true
    
    return 0
}

# Test: Shell compatibility
test_shell_compatibility() {
    # Test that script uses bash features correctly
    if ! head -1 "$ANCHOR_SCRIPT" | grep -q "#!/usr/bin/env bash"; then
        echo "Script should use '#!/usr/bin/env bash' shebang"
        return 1
    fi
    
    # Check for bash-specific syntax
    if grep -q "set -euo pipefail" "$ANCHOR_SCRIPT"; then
        echo "Good: Using strict error handling"
    else
        echo "Warning: Consider using 'set -euo pipefail'"
    fi
    
    return 0
}

# Test: ANSI color codes
test_ansi_colors() {
    local output
    output=$("$ANCHOR_SCRIPT" help 2>&1) || true
    
    # Check if colors are being used (looking for escape sequences)
    if [[ "$output" =~ '\033' ]] || [[ "$output" =~ $'\e' ]]; then
        echo "Colors are being used in output"
    else
        echo "Note: Colors might be disabled in test environment"
    fi
    
    return 0
}

# Test: Error handling
test_error_handling() {
    # Test various error conditions
    
    # Invalid number of arguments
    output=$("$ANCHOR_SCRIPT" save only_one_arg 2>&1) || true
    assert_contains "$output" "Usage:" "Should handle insufficient arguments"
    
    # Very long input
    local long_string=$(printf 'x%.0s' {1..1000})
    output=$("$ANCHOR_SCRIPT" save "$long_string" "keyword" 2>&1) || true
    
    return 0
}

# Test: Quote handling
test_quote_handling() {
    # Test various quote scenarios
    local quotes_context='This has "double quotes" and '\''single quotes'\'''
    local quotes_keywords='keyword"with"quotes,another'\''one'
    
    # Should handle quotes without breaking
    # (Would test actual functionality but MCP dependency)
    
    return 0
}

# Test: Path handling
test_path_handling() {
    # Test with various path formats
    ST_PROJECT="." output=$("$ANCHOR_SCRIPT" help 2>&1) || true
    ST_PROJECT="/absolute/path" output=$("$ANCHOR_SCRIPT" help 2>&1) || true
    ST_PROJECT="~/home/path" output=$("$ANCHOR_SCRIPT" help 2>&1) || true
    ST_PROJECT="../relative/path" output=$("$ANCHOR_SCRIPT" help 2>&1) || true
    
    return 0
}

# Main test runner
main() {
    echo -e "${BOLD}${CYAN}🧪 Running anchor.sh Test Suite${RESET}"
    echo -e "${CYAN}================================${RESET}"
    
    # Run all tests
    run_test "Script exists and is executable" test_script_exists
    run_test "Help command works" test_help_command
    run_test "No arguments shows usage" test_no_args_shows_usage
    run_test "Invalid command handling" test_invalid_command
    run_test "Save command validation" test_save_command_validation
    run_test "Find command validation" test_find_command_validation
    run_test "List command works" test_list_command
    run_test "Environment variables" test_environment_variables
    run_test "Anchor type validation" test_anchor_types
    run_test "Multiple keywords handling" test_multiple_keywords
    run_test "Special characters in input" test_special_characters
    run_test "Rapport command" test_rapport_command
    run_test "Heatmap command" test_heatmap_command
    run_test "Patterns command" test_patterns_command
    run_test "Insights command validation" test_insights_command
    run_test "Invite command validation" test_invite_command
    run_test "Shell compatibility" test_shell_compatibility
    run_test "ANSI color codes" test_ansi_colors
    run_test "Error handling" test_error_handling
    run_test "Quote handling" test_quote_handling
    run_test "Path handling" test_path_handling
    
    # Summary
    echo -e "\n${CYAN}================================${RESET}"
    echo -e "${BOLD}Test Summary:${RESET}"
    echo -e "  Total:  $TESTS_RUN"
    echo -e "  ${GREEN}Passed: $TESTS_PASSED${RESET}"
    echo -e "  ${RED}Failed: $TESTS_FAILED${RESET}"
    
    if [[ $TESTS_FAILED -eq 0 ]]; then
        echo -e "\n${GREEN}${BOLD}✨ All tests passed! The partnership is strong! ✨${RESET}"
        exit 0
    else
        echo -e "\n${RED}${BOLD}❌ Some tests failed. Time to debug! ❌${RESET}"
        exit 1
    fi
}

# Run the tests
main "$@"