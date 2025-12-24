# Dashboard development workflow
# Usage:
#   make dev                - Start development server with hot-reload
#   make build              - Production build  
#   make test [SCENARIO=x]  - Generate test data and serve
#   make clean              - Clean build artifacts

.PHONY: dev build test test-data clean

# Output directory for dx builds
DX_OUTPUT := target/dx/git-bench-dashboard/debug/web/public

# Development with hot-reload
dev:
	dx serve

# Production build
build:
	dx build --release
	@echo "✅ Built to $(DX_OUTPUT)"

# Generate test data (optional: SCENARIO=simple|new-bench|renamed|regression|deep-hierarchy|full)
SCENARIO ?= full
test-data:
	cargo run --bin test_data_generator -- $(SCENARIO)
	@mkdir -p $(DX_OUTPUT)
	cp test-data.json $(DX_OUTPUT)/data.json
	@echo "✅ Test data ($(SCENARIO)) copied to $(DX_OUTPUT)/data.json"

# Generate test data and start dev server
test: test-data
	dx serve

# Clean build artifacts
clean:
	rm -rf target/dx
	rm -f test-data.json
	dx clean 2>/dev/null || true

# Copy real benchmark data to output
update-data:
	@mkdir -p $(DX_OUTPUT)
	cp ../../benchmark-data.json $(DX_OUTPUT)/data.json
	@echo "✅ Data updated. Refresh browser."

# Quick shortcuts
.PHONY: simple regression hierarchy
simple:
	$(MAKE) test SCENARIO=simple
regression:
	$(MAKE) test SCENARIO=regression
hierarchy:
	$(MAKE) test SCENARIO=deep-hierarchy

