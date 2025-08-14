import "encoding/toml"

// Test data with mixed route types
data: {
    trigger: http: [
        {route: "/...", component: "gateway"},
        {route: {private: true}, component: "tool"},
    ]
}

// Try to encode as TOML string to see output
output: toml.Marshal(data)
