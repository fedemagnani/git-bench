# git-bench local development

.PHONY: demo bench store build dev serve clean refresh

# Full end-to-end demo
demo: bench store build serve

# Run benchmarks and save output
bench:
	cargo bench 2>&1 | tee benchmark-output.txt

# Store results using git-bench CLI
store:
	cargo run -p git-bench -- store \
		--output-file benchmark-output.txt \
		--name "git-bench" \
		--data-file benchmark-data.json

# Build dashboard and copy to dist/
build:
	cd crates/dashboard && dx build --release
	rm -rf dist
	cp -r crates/dashboard/target/dx/git-bench-dashboard/release/web/public dist
	cp benchmark-data.json dist/data.json

# Development: dx serve with data
dev: store
	@mkdir -p crates/dashboard/assets
	cp benchmark-data.json crates/dashboard/assets/data.json
	cd crates/dashboard && dx serve

# Serve static build
serve:
	@echo "Dashboard: http://localhost:8080"
	cd dist && python3 -m http.server 8080

# Clean build artifacts
clean:
	rm -f benchmark-output.txt benchmark-data.json
	rm -rf dist

# Quick refresh: re-run benchmarks and update data for dx serve
refresh: bench store
	@mkdir -p crates/dashboard/assets
	cp benchmark-data.json crates/dashboard/assets/data.json

