# git-bench local development

.PHONY: demo bench store dev serve clean refresh

# Full end-to-end demo
demo: bench store dev

# Run benchmarks and save output
bench:
	cargo bench 2>&1 | tee benchmark-output.txt

# Store results using git-bench CLI
store:
	cargo run -p git-bench -- store \
		--output-file benchmark-output.txt \
		--name "git-bench" \
		--data-file benchmark-data.json

# Development: dx serve with data
dev: store
	@mkdir -p crates/dashboard/target/dx/git-bench-dashboard/debug/web/public
	cp benchmark-data.json crates/dashboard/target/dx/git-bench-dashboard/debug/web/public/data.json
	cd crates/dashboard && dx serve

# Serve static build (after dx build --release)
serve:
	@echo "Dashboard: http://localhost:8080"
	cd crates/dashboard/dist && python3 -m http.server 8080

# Clean build artifacts
clean:
	rm -f benchmark-output.txt
	rm -f benchmark-data.json

# Quick refresh: re-run benchmarks and update data for dx serve
refresh: bench store
	cp benchmark-data.json crates/dashboard/target/dx/git-bench-dashboard/debug/web/public/data.json

