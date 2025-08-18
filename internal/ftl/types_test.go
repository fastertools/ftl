package ftl

import (
	"testing"
)

func TestApplicationValidation(t *testing.T) {
	tests := []struct {
		name    string
		app     Application
		wantErr bool
	}{
		{
			name: "valid application",
			app: Application{
				Name:    "test-app",
				Version: "1.0.0",
				Access:  AccessPublic,
			},
			wantErr: false,
		},
		{
			name: "missing name",
			app: Application{
				Version: "1.0.0",
			},
			wantErr: true,
		},
		{
			name: "invalid name format",
			app: Application{
				Name:    "Test_App", // uppercase and underscore not allowed
				Version: "1.0.0",
			},
			wantErr: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			tt.app.SetDefaults()
			err := tt.app.Validate()
			if (err != nil) != tt.wantErr {
				t.Errorf("Validate() error = %v, wantErr %v", err, tt.wantErr)
			}
		})
	}
}

func TestComponentSourceMarshaling(t *testing.T) {
	comp := Component{
		ID:     "test",
		Source: LocalSource("./test.wasm"),
	}

	// Test that local source is correctly identified
	if !comp.Source.IsLocal() {
		t.Error("Expected local source")
	}

	if comp.Source.GetPath() != "./test.wasm" {
		t.Errorf("Expected path ./test.wasm, got %s", comp.Source.GetPath())
	}
}
