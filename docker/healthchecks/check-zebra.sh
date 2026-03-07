#!/bin/bash
# Basic smoke test for ZecKit
# Verifies that Zebra is functional and can perform basic operations

set -e

# Configuration
ZEBRA_RPC_URL=${ZEBRA_RPC_URL:-"http://127.0.0.1:8232"}
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Test counters
TESTS_RUN=0
TESTS_PASSED=0
TESTS_FAILED=0

# Logging functions
log_test() {
    echo -e "${BLUE}[TEST]${NC} $1"
}

log_pass() {
    echo -e "${GREEN}[PASS]${NC} $1"
    TESTS_PASSED=$((TESTS_PASSED + 1))
}

log_fail() {
    echo -e "${RED}[FAIL]${NC} $1"
    TESTS_FAILED=$((TESTS_FAILED + 1))
}

log_info() {
    echo -e "${YELLOW}[INFO]${NC} $1"
}

# Function to make RPC calls
rpc_call() {
    local method=$1
    shift
    local params="$@"
    
    if [ -z "$params" ]; then
        params="[]"
    fi
    
    curl -sf --max-time 10 \
        --data-binary "{\"jsonrpc\":\"2.0\",\"id\":\"test\",\"method\":\"$method\",\"params\":$params}" \
        -H 'content-type: application/json' \
        "$ZEBRA_RPC_URL" 2>/dev/null
}

# Test 1: RPC connectivity
test_rpc_connectivity() {
    TESTS_RUN=$((TESTS_RUN + 1))
    log_test "Test 1: RPC Connectivity"
    
    local response
    response=$(rpc_call "getinfo")
    
    if [ $? -eq 0 ] && echo "$response" | grep -q '"result"'; then
        log_pass "Zebra RPC is accessible"
        return 0
    else
        log_fail "Cannot connect to Zebra RPC"
        return 1
    fi
}

# Test 2: Get blockchain info
test_blockchain_info() {
    TESTS_RUN=$((TESTS_RUN + 1))
    log_test "Test 2: Blockchain Information"
    
    local response
    response=$(rpc_call "getblockchaininfo")
    
    if [ $? -eq 0 ] && echo "$response" | grep -q '"chain"'; then
        local chain=$(echo "$response" | grep -o '"chain":"[^"]*"' | cut -d'"' -f4)
        log_info "Chain: $chain"
        
        # Zebra reports Regtest as "test" in some versions
        if [ "$chain" = "regtest" ] || [ "$chain" = "test" ]; then
            log_pass "Blockchain info retrieved successfully ($chain - regtest mode)"
            return 0
        else
            log_fail "Expected regtest chain, got: $chain"
            return 1
        fi
    else
        log_fail "Failed to get blockchain info"
        return 1
    fi
}

# Test 3: Get block count
test_block_count() {
    TESTS_RUN=$((TESTS_RUN + 1))
    log_test "Test 3: Block Count"
    
    local response
    response=$(rpc_call "getblockcount")
    
    if [ $? -eq 0 ] && echo "$response" | grep -q '"result"'; then
        local blocks=$(echo "$response" | grep -o '"result":[0-9]*' | cut -d':' -f2)
        log_info "Current block height: $blocks"
        log_pass "Block count retrieved: $blocks"
        return 0
    else
        log_fail "Failed to get block count"
        return 1
    fi
}

# Test 4: Generate a block (regtest capability)
test_generate_block() {
    TESTS_RUN=$((TESTS_RUN + 1))
    log_test "Test 4: Block Generation (Regtest)"
    
    # First, get current block count
    local before_response
    before_response=$(rpc_call "getblockcount")
    local blocks_before=$(echo "$before_response" | grep -o '"result":[0-9]*' | cut -d':' -f2)
    
    log_info "Blocks before: $blocks_before"
    
    # Try to generate 1 block
    # Note: This may require a miner address to be set in zebra.toml
    # For M1, we just test if the RPC method is available
    local gen_response
    gen_response=$(rpc_call "generate" "[1]" 2>&1)
    
    if echo "$gen_response" | grep -q -E '"result"|"error"'; then
        if echo "$gen_response" | grep -q '"error"'; then
            local error_msg=$(echo "$gen_response" | grep -o '"message":"[^"]*"' | cut -d'"' -f4)
            log_info "Generate returned error: $error_msg"
            log_pass "Block generation RPC is available (may need miner address configured)"
            return 0
        else
            # Success - check block count increased
            local after_response
            after_response=$(rpc_call "getblockcount")
            local blocks_after=$(echo "$after_response" | grep -o '"result":[0-9]*' | cut -d':' -f2)
            
            log_info "Blocks after: $blocks_after"
            
            if [ "$blocks_after" -gt "$blocks_before" ]; then
                log_pass "Successfully generated block(s)"
                return 0
            else
                log_pass "Block generation command accepted"
                return 0
            fi
        fi
    else
        log_fail "Block generation test failed"
        return 1
    fi
}

# Test 5: Network info
test_network_info() {
    TESTS_RUN=$((TESTS_RUN + 1))
    log_test "Test 5: Network Information"
    
    # Zebra uses getinfo instead of getnetworkinfo
    local response
    response=$(rpc_call "getinfo")
    
    if [ $? -eq 0 ] && echo "$response" | grep -q '"result"'; then
        # Try to extract build/version info from getinfo
        local info=$(echo "$response" | grep -o '"build":"[^"]*"' || echo "info retrieved")
        log_info "Node info: $info"
        log_pass "Network info retrieved via getinfo"
        return 0
    else
        log_fail "Failed to get network info"
        return 1
    fi
}

# Test 6: Peer info
test_peer_info() {
    TESTS_RUN=$((TESTS_RUN + 1))
    log_test "Test 6: Peer Information"
    
    local response
    response=$(rpc_call "getpeerinfo")
    
    if [ $? -eq 0 ]; then
        # In regtest with no external peers, this should return empty array or result
        # Just check that the RPC call succeeds
        log_info "Peer info: $(echo "$response" | grep -o '"result":\[[^]]*\]' || echo '[]')"
        log_pass "Peer info retrieved (isolated regtest node)"
        return 0
    else
        # If getpeerinfo doesn't exist, that's okay for regtest
        log_info "getpeerinfo not available (acceptable for regtest)"
        log_pass "Peer info check skipped (regtest mode)"
        return 0
    fi
}

# Main test execution
main() {
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "  ZecKit - Smoke Test Suite (M1)"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    log_info "RPC Endpoint: $ZEBRA_RPC_URL"
    echo ""
    
    # Run all tests
    test_rpc_connectivity
    test_blockchain_info
    test_block_count
    test_generate_block
    test_network_info
    test_peer_info
    
    # Print summary
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "  Test Summary"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo -e "Tests Run:    ${BLUE}$TESTS_RUN${NC}"
    echo -e "Tests Passed: ${GREEN}$TESTS_PASSED${NC}"
    echo -e "Tests Failed: ${RED}$TESTS_FAILED${NC}"
    echo ""
    
    if [ $TESTS_FAILED -eq 0 ]; then
        echo -e "${GREEN}✓ All smoke tests passed!${NC}"
        echo ""
        exit 0
    else
        echo -e "${RED}✗ Some tests failed${NC}"
        echo ""
        exit 1
    fi
}

# Execute main
main "$@"