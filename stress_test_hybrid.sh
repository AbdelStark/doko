#!/bin/bash

# Hybrid Vault Stress Test
echo "🧪 HYBRID VAULT STRESS TEST"
echo "=========================="

SUCCESS_COUNT=0
FAILURE_COUNT=0
TOTAL_TESTS=10

for i in $(seq 1 $TOTAL_TESTS); do
    echo -n "Test $i/$TOTAL_TESTS: "
    
    # Run the test with timeout
    if timeout 30s cargo run --quiet -- auto-demo --vault-type hybrid --scenario cold-recovery --amount 10000 >/dev/null 2>&1; then
        echo "✅ SUCCESS"
        ((SUCCESS_COUNT++))
    else
        echo "❌ FAILED"
        ((FAILURE_COUNT++))
    fi
done

echo
echo "📊 RESULTS:"
echo "==========="
echo "✅ Successes: $SUCCESS_COUNT/$TOTAL_TESTS ($(echo "scale=1; $SUCCESS_COUNT * 100 / $TOTAL_TESTS" | bc)%)"
echo "❌ Failures:  $FAILURE_COUNT/$TOTAL_TESTS ($(echo "scale=1; $FAILURE_COUNT * 100 / $TOTAL_TESTS" | bc)%)"

if [ $SUCCESS_COUNT -eq $TOTAL_TESTS ]; then
    echo "🎉 PERFECT: 100% success rate achieved!"
    exit 0
elif [ $SUCCESS_COUNT -gt $((TOTAL_TESTS / 2)) ]; then
    echo "⚠️  IMPROVED: Success rate > 50% but not perfect"
    exit 1
else
    echo "💥 CRITICAL: Success rate < 50%, major issues remain"
    exit 2
fi