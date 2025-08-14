// Direct CUE input example
// This demonstrates defining an FTL application directly in CUE

// Define the application using the FTL schema
app: {
	name:        "cue-platform"
	version:     "2.0.0"
	description: "Platform defined directly in CUE"
	
	tools: [
		{
			id: "geo"
			source: {
				registry: "ghcr.io"
				package:  "bowlofarugula/geo"
				version:  "0.0.1"
			}
			environment: {
				LOG_LEVEL:    "debug"
				MAX_WORKERS:  "8"
				CACHE_SIZE:   "1024"
			}
		},
		{
			id: "fluid"
			source: {
				registry: "ghcr.io"
				package:  "bowlofarugula/fluid"
				version:  "0.0.1"
			}
			environment: {
				PRECISION:       "double"
				SOLVER_TYPE:     "PISO"
				CONVERGENCE:     "1e-6"
			}
		},
		{
			id: "analyzer"
			source: "./analyzer.wasm"
			build: {
				command: "cargo build --target wasm32-wasi --release"
				watch: ["src/**/*.rs", "Cargo.toml"]
			}
			environment: {
				ANALYSIS_MODE: "comprehensive"
			}
		},
	]
	
	// Private access with WorkOS authentication
	access: "private"
	auth: {
		provider:     "workos"
		org_id:       "org_cue_12345"
		jwt_issuer:   "https://api.workos.com"
		jwt_audience: "cue-platform"
	}
}